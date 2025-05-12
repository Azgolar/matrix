use getopts::Options;
use std::process;

/*
    Parsen der übergebenen CLI Argumente
*/
pub fn verarbeiten(nutzer: &[String]) -> Result<(Vec<u32>, String, bool), String> {
    
    // getopt Optionen
    let mut optionen: Options = Options::new();
    optionen.optopt("n", "", "", "");
    optionen.optopt("c", "", "", "");
    optionen.optflag("d", "", "");
    optionen.optflag("h", "", "");

    // getopt parsen
    let eingabe: getopts::Matches = optionen.parse(nutzer)
        .map_err(|_| "Fehler beim Parsen der Eingabe. Benutzung siehe -h".to_string())?;

    // ausgeben der Hilfe
    if eingabe.opt_present("h") {
        println!(
            "\nPflicht:\n\
             -n <a,b>   erzeuge Matrizen im Bereich [a,b]\n\
             \nOptional:\n\
             -c <datei> Ergebnisdatei, Default: matrix.txt\n\
             -d         Debugmodus\n\
             -h         Hilfe\n"
        );
        process::exit(0);
    }

    // -n <a,b>
    let n_str: String = eingabe.opt_str("n").ok_or_else(|| "-n fehlt. Benutzung siehe -h".to_string())?;
    let n: Vec<u32> = konvertieren(&n_str).map_err(|_| "-n hat falsches Format. Benutzung siehe -h".to_string())?;

    // -c <datei> (Default matrix.txt)
    let mut datei: String = eingabe.opt_str("c").unwrap_or_else(|| "matrix.txt".into());
    if !datei.ends_with(".txt") {
        datei = datei + ".txt";
    }

    // -d
    let debug: bool = eingabe.opt_present("d");

    Ok((n, datei, debug))
}

/*
    Konvertiert einen durch Komma getrennten String "a,b" in einen integer Vektor, wobei
    die Schrittweite adaptiv an die Eingabegröße angepasst wird
*/
fn konvertieren(range: &str) -> Result<Vec<u32>, ()> {
    let mut ergebnis: Vec<u32> = Vec::new();

    // String teilen
    let geteilt: (&str, &str) = range.split_once(',').ok_or(())?;
    let a: u32 = geteilt.0.trim().parse::<u32>().map_err(|_| ())?;
    let b: u32 = geteilt.1.trim().parse::<u32>().map_err(|_| ())?;

    if a > b {
        return Err(());
    }

    // Schrittweite adaptiv an die Größe anpasen
    ergebnis.push(a);
    let mut letztes = a;
    while letztes < b {
        let schritt: u32 = match letztes {
            0..=9       => 2,
            10..=99     => 10,
            100..=999   => 100,
            1000..=9999 => 500,
            _           => 1000,
        };
        let next = letztes.saturating_add(schritt);
        if next >= b {
            ergebnis.push(b);
            break;
        }
        ergebnis.push(next);
        letztes = next;
    }
    Ok(ergebnis)
}

/// Speichert die Prozessorinformationen, Eingabegrößen n und die Laufzeiten in eine Datei
pub fn speichern(datei: &str, laufzeit: &Vec<f64>, threads: usize) 
{
    // cpuinfo einlesen
    let cpuinfo: String = std::fs::read_to_string("/proc/cpuinfo")
        .expect("\nFehler beim Lesen von /proc/cpuinfo\n");

    // name
    let name = cpuinfo.lines().find(|l| l.starts_with("model name"))
        .and_then(|l| l.splitn(2, ':').nth(1)).map(str::trim).unwrap_or("").to_string();

    // logisch
    let logisch: u32 = core_affinity::get_core_ids().map_or(0, |ids| ids.len() as u32);

    // physisch
    let physisch: u32 = cpuinfo.lines().find(|l| l.starts_with("cpu cores"))
        .and_then(|l| l.splitn(2, ':').nth(1)).and_then(|v| v.trim().parse::<u32>().ok()).unwrap_or(0);
 
    let mut hyperthreading: u32;
    if physisch > 0 { 
        hyperthreading = logisch / physisch; 
    } 
    else { 
        hyperthreading = 0;
    };

    if name.is_empty() || logisch == 0 || physisch == 0 || hyperthreading == 0 {
        eprintln!("Fehler beim Lesen der Prozessor­spezifikationen");
    }

    let infos: String = format!("{},{},{},{}", name, logisch, physisch, hyperthreading);
}