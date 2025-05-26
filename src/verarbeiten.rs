use getopts::Options;
use std::{process, io::Write, fs::read_to_string, fs::OpenOptions, path::Path};
use core_affinity::{CoreId, get_core_ids};

pub struct ProzessorSpecs {
    pub name: String,
    pub logisch: u32,
    pub physisch: u32,
    pub hyperthreading: u32
}



/*
    Parsen der übergebenen CLI Argumente
*/
pub fn eingabe(nutzer: &[String]) -> (Vec<u32>, u32, String, bool) {
    // getopt Optionen
    let mut optionen: Options = Options::new();
    optionen.optopt("n", "", "", "");
    optionen.optopt("b", "", "", "");
    optionen.optopt("c", "", "", "");
    optionen.optflag("d", "", "");
    optionen.optflag("h", "", "");

    // getopt parsen
    let eingabe: getopts::Matches = optionen
        .parse(nutzer)
        .unwrap_or_else(|_| {
            println!("\nFehler beim Parsen der Eingabe. Benutzung siehe -h\n");
            std::process::exit(1);
        });

    // ausgeben der Hilfe
    if eingabe.opt_present("h") {
                println!("
        \nPflicht:
        -n <zahl>   Matrixgrößen im Bereich [6, zahl] erzeugen
        -b <zahl>   1: regulär parallel
                    2: loop unrolling
                    3: block tiling
                    4: rayon
                    5: crossbeam
        Optional:
        -c <datei>  Ergebnisdatei, Default: matrix.txt
        -d          Debugmodus\n
        ");
        process::exit(0);
    }

    // parsen von: n <a>, Default n = 30
    let n: String = eingabe.opt_str("n").unwrap_or_else(|| { fehlerausgabe("Parameter n fehlt"); });     
    let n: Vec<u32> = konvertieren(6, &n);    

    let b: u32 = eingabe.opt_str("b").unwrap_or_else(|| { fehlerausgabe("Parameter b fehlt"); })
        .parse::<u32>()
        .unwrap_or_else(|_| { fehlerausgabe("Parameter b muss eine Zahl sein");});

    // gültiger Bereich
    if !(1..=5).contains(&b) { fehlerausgabe("Parameter b muss eine Zahl zwischen 1 und 5 sein"); }                              

    // parsen von: c <datei>, Default datei = matrix.txt
    let mut datei: String = eingabe.opt_str("c").unwrap_or_else(|| "matrix.txt".into());
    if !datei.ends_with(".txt") {
        datei = datei + ".txt";
    }

    // parsen von d
    let debug: bool = eingabe.opt_present("d");

    (n, b, datei, debug)
}

fn fehlerausgabe(fehler: &str) -> ! {
        println!("\nFehler! {}. Benutzung siehe -h\n", fehler);
        process::exit(1);
}

/*
    Erzeugt einn Vektor mit Zahlen im Bereich [anfang, ende]
    Die Schrittweite adaptiv größer
*/
fn konvertieren(anfang: u32, ende: &str) -> Vec<u32> {
    let mut ergebnis: Vec<u32> = Vec::new();

    let ende: u32 = ende.trim().parse::<u32>().unwrap_or_else(|_| { fehlerausgabe("Format für n <ganze Zahl>\n"); });

    if anfang >= ende {
        fehlerausgabe(&format!("-n <zahl> muss größer {} sein", anfang));
    }

    // Schrittweite adaptiv an die Größe anpasen
    ergebnis.push(anfang);
    let mut letztes: u32 = anfang;
    while letztes < ende {
        let schritt: u32 = match letztes {
            0..=9       => 4,
            10..=99     => 6,
            100..=999   => 100,
            1000..=9999 => 500,
            _           => 1000,
        };
        let nächstes: u32 = letztes.saturating_add(schritt);
        if nächstes >= ende {
            ergebnis.push(ende);
            break;
        }
        ergebnis.push(nächstes);
        letztes = nächstes;
    }
    ergebnis
}

/*
    setzt die Prozessorspezifikationen
*/
impl ProzessorSpecs {
    pub fn new() -> Self {
        let mut daten = ProzessorSpecs {
            name: String::new(),
            logisch: 0,
            physisch: 0,
            hyperthreading: 0,
        };

        let cpuinfo = read_to_string("/proc/cpuinfo")
            .unwrap_or_default();

        for line in cpuinfo.lines() {
            if daten.name.is_empty() && line.starts_with("model name") {
                if let Some(val) = line.splitn(2, ':').nth(1) {
                    daten.name = val.trim().to_string();
                }
            }
            else if daten.logisch == 0 && line.starts_with("siblings") {
                if let Some(val) = line.splitn(2, ':').nth(1) {
                    daten.logisch = val.trim().parse().unwrap_or(0);
                }
            }
            else if daten.physisch == 0 && line.starts_with("cpu cores") {
                if let Some(val) = line.splitn(2, ':').nth(1) {
                    daten.physisch = val.trim().parse().unwrap_or(0);
                }
            }
            if !daten.name.is_empty() && daten.logisch != 0 && daten.physisch != 0 {
                break;
            }
        }

        if daten.physisch > 0 {
            daten.hyperthreading = daten.logisch / daten.physisch;
        } 
        else {
            daten.hyperthreading = 1;
        };

        if daten.name.is_empty() || daten.logisch == 0 || daten.physisch == 0 {
            println!("Fehler beim Auslesen der Prozessordaten");
        }

        daten
    }
}

/// Liefert bis zu `anzahl` CoreId in phys+HT-Reihenfolge
pub fn pinnen_liste(anzahl: usize, prozessor: &ProzessorSpecs) -> Vec<CoreId> {
    // 1) Alle Cores einlesen
    let kern_ids: Vec<CoreId> = get_core_ids().unwrap_or_else(|| {
        eprintln!("Fehler beim Lesen der Core-IDs für CPU-Pinning");
        process::exit(1);
    });

    let mut reihenfolge: Vec<CoreId> = Vec::with_capacity(anzahl);

    // 2) Physische Kerne zuerst hinzufügen
    for i in 0..prozessor.physisch as usize {
        if reihenfolge.len() >= anzahl { 
            break;
        }
        
        let index: usize = i * prozessor.hyperthreading as usize;
        let gefunden: Option<&CoreId> = kern_ids.iter().find(|c| c.id == index);
        reihenfolge.push(*gefunden.unwrap());
    }

    // 3) Restliche (logische) Kerne
    for i in kern_ids.iter() {
        if reihenfolge.len() >= anzahl {
            break;
        }
        // nur hinzufügen, wenn noch nicht drin
        if !reihenfolge.iter().any(|c| c.id == i.id) {
            reihenfolge.push(*i);
        }
    }

    reihenfolge
}


/// Speichert die Prozessorinformationen, Eingabegrößen n und die Laufzeiten in eine Datei
pub fn speichern(datei: &str, n: &Vec<u32>, laufzeit: &Vec<f64>, threads: usize, prozessor: &ProzessorSpecs) {
     // Prüfen, ob die Datei bereits existiert
    let existiert = Path::new(datei).exists();

    // Datei im Append-Modus öffnen (wird erstellt, falls sie nicht existiert)
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(datei)
        .unwrap_or_else(|e| {
            eprintln!("Fehler beim Öffnen der Datei {}: {}", datei, e);
            process::exit(1);
        });

    // Kopfzeile nur schreiben, wenn die Datei gerade erst angelegt wurde
    if !existiert {
        writeln!(file, "{},{},{},{}", prozessor.name, prozessor.logisch, prozessor.physisch, prozessor.hyperthreading)
            .unwrap_or_else(|_| {
                println!("Fehler beim Schreiben der Prozessorinformationen");
                process::exit(1);
            });
    }

    // Messdaten anhängen
    for (&größe, &zeit) in n.iter().zip(laufzeit.iter()) {
        writeln!(file, "{},{},{}", threads, größe, zeit).unwrap_or_else(|_| {
            println!("Fehler beim Schreiben der Daten");
            process::exit(1);
        });
    }
}
