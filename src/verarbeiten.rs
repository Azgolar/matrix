use getopts::Options;
use std::{process, io::Write, fs::OpenOptions, path::Path};

/*
    Parsen der übergebenen CLI Argumente
*/
pub fn eingabe(nutzer: &[String]) -> (Vec<u32>, String, bool) {
    // getopt Optionen
    let mut optionen: Options = Options::new();
    optionen.optopt("n", "", "", "");
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
        println!(
             "\nOptional:\n\
             -n <zahl>  Matrixgrößen im Bereich [6, zahl] erzeugen\n\
             -c <datei> Ergebnisdatei, Default: matrix.txt\n\
             -d         Debugmodus\n"
        );
        process::exit(0);
    }

    // parsen von: n <a>, Default n = 30
    let n: String = eingabe.opt_str("n").unwrap_or("30".to_string());          // noch ändern
    let n: Vec<u32> = konvertieren(6, &n);                                   // noch ändern

    // parsen von: c <datei>, Default datei = matrix.txt
    let mut datei: String = eingabe.opt_str("c").unwrap_or_else(|| "matrix.txt".into());
    if !datei.ends_with(".txt") {
        datei = datei + ".txt";
    }

    // parsen von d
    let debug: bool = eingabe.opt_present("d");

    (n, datei, debug)
}


/*
    Erzeugt einn Vektor mit Zahlen im Bereich [anfang, ende]
    Die Schrittweite adaptiv größer
*/
fn konvertieren(anfang: u32, ende: &str) -> Vec<u32> {
    let mut ergebnis: Vec<u32> = Vec::new();

    let ende: u32 = ende.trim().parse::<u32>().unwrap_or_else(|_| {
            println!("\nFehler. Format für n <ganze Zahl>\n");
            std::process::exit(1);
        });


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

/// Speichert die Prozessorinformationen, Eingabegrößen n und die Laufzeiten in eine Datei
pub fn speichern(datei: &str, n: &Vec<u32>, laufzeit: &Vec<f64>, threads: usize) {
    // cpuinfo einlesen
    let cpuinfo: String =
        std::fs::read_to_string("/proc/cpuinfo").expect("\nFehler beim Lesen von /proc/cpuinfo\n");

    // name
    let name = cpuinfo
        .lines()
        .find(|l| l.starts_with("model name"))
        .and_then(|l| l.splitn(2, ':').nth(1))
        .map(str::trim)
        .unwrap_or("")
        .to_string();

    // logisch
    let logisch: u32 = core_affinity::get_core_ids().map_or(0, |ids| ids.len() as u32);

    // physisch
    let physisch: u32 = cpuinfo
        .lines()
        .find(|l| l.starts_with("cpu cores"))
        .and_then(|l| l.splitn(2, ':').nth(1))
        .and_then(|v| v.trim().parse::<u32>().ok())
        .unwrap_or(0);

    let hyperthreading: u32;
    if physisch > 0 {
        hyperthreading = logisch / physisch;
    } else {
        hyperthreading = 0;
    };

    if name.is_empty() || logisch == 0 || physisch == 0 || hyperthreading == 0 {
        println!("Fehler beim Lesen der Prozessor­spezifikationen\n");
    }

    let existiert: bool = Path::new(datei).exists();

    // Datei öffnen (wird überschrieben)
    let path: &Path = Path::new(datei);
    let mut file: std::fs::File = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .unwrap_or_else(|e| {
            println!("Fehler beim Öffnen der Datei {}: {}\n", datei, e);
            std::process::exit(1);
        });
        

    // Prozessor Information in erste Zeile schreiben
    let kopf: Result<(), std::io::Error> = writeln!(file, "{},{},{},{}", name, logisch, physisch, hyperthreading);
    if kopf.is_err() {
        println!("Fehler beim Schreiben der Prozessorinformationen\n");
    }

    // jede Messung in eine neue Zeile
    for (&i, &j) in n.iter().zip(laufzeit.iter()) {
        let fehler: Result<(), std::io::Error> = writeln!(file, "{},{},{}", threads, i, j);
        if fehler.is_err() {
            println!("Fehler beim Schreiben der Daten\n");
            std::process::exit(1);
        } 
    }

    if existiert {
        println!("Daten erfolgreich geschrieben. {} wurde überschrieben.", datei);
    }
    else {
        println!("Daten erfolgreich in {} geschrieben\n", datei);
    }
}
