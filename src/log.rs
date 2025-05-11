use std::{fs::{File, read_to_string}, io::Write};
use core_affinity;

struct ProzessorSpecs 
{
    name: String,
    logisch: u32,
    physisch: u32,
    threads: u32,
}

impl ProzessorSpecs 
{
    pub fn new() -> Self 
    {
        // auslesen der Prozessorspezifikationen
        let cpuinfo: String = read_to_string("/proc/cpuinfo").unwrap();

        // Modellname
        let name: String = cpuinfo.lines().find(|l| l.starts_with("model name"))
            .and_then(|l| l.splitn(2, ':').nth(1)).map(str::trim).unwrap_or("").to_string();

        // Anzahl logischer Kerne
        let logisch: u32 = core_affinity::get_core_ids().map_or(0, |ids| ids.len() as u32);

        // Anzahl physischer Kerne
        let physisch: u32 = cpuinfo.lines().find(|l| l.starts_with("cpu cores"))
            .and_then(|l| l.splitn(2, ':').nth(1)).and_then(|v| v.trim().parse::<u32>().ok())
            .unwrap_or(0);

        // Anzahl Threads pro physischem Kern
        let threads: u32;
        if physisch > 0 
        {
            threads = logisch / physisch
        } 
        else
        {
            threads = 0;
        };

        // Fehlerprüfung beim Auslesen
        if name.is_empty() || logisch == 0 || physisch == 0 || threads == 0 
        {
            println!("Fehler beim Lesen der Prozessor­spezifikationen");
        }

        ProzessorSpecs { name, logisch, physisch, threads }
    }
}

/// Speichert die Prozessorinformationen, Eingabegrößen n und die Laufzeiten in eine Datei
pub fn speichern(name: &str, n: &str, laufzeit: &str) 
{

    // Prozessorinformationen
    let prozessor: ProzessorSpecs = ProzessorSpecs::new();
    let infos: String = format!("{},{},{},{}", prozessor.name, prozessor.logisch, prozessor.physisch, prozessor.threads);

    // Stings in Vektor umwandeln
    let n: Vec<u32> = n.split(',').map(str::trim).map(|a| a.parse().unwrap()).collect();
    let laufzeit: Vec<f64> = laufzeit.split(',').map(str::trim).map(|a| a.parse().unwrap()).collect();

    // erstellen der Zeilen
    let mut zeilen: Vec<String> = n.iter().zip(laufzeit.iter()).map(|(a, b)| format!("{},{}", a, b)).collect();

    // Prozessorinformationen zuerst
    zeilen.insert(0, infos.to_string());

    // Inhalt zusammenbauen
    let inhalt: String = zeilen.join("\n");

    // In Datei schreiben (existierende Datei überschreiben)
    match File::create(name) 
    {
        Ok(mut datei) => 
        {
            if let Err(f) = datei.write_all(inhalt.as_bytes()) 
            {
                eprintln!("Fehler beim Schreiben: {}", f);
            }
        }
        Err(f) => eprintln!("Fehler beim Öffnen der Datei: {}", f),
    }
}