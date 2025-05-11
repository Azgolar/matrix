use getopts::Options;
use core_affinity;

#[derive(Debug)]
pub struct Settings 
{
    pub kerne: Vec<u32>,    // Kerne für das Pinning
    pub n: Vec<u32>,        // Eingabegrößen für Benchmarking
    pub threads: u32,       // Anzahl der Threads
    pub log: String,        // Name der Logdatei
    pub flagge: bool,       // Debug-Flagge
}

pub fn verarbeiten(eingabe: &[String]) -> Settings
{
    // getopt Einstellungen
    let mut parameter: Options = Options::new();
    // Pflichtparameter
    parameter.optopt("a", "","", "");
    parameter.optopt("b", "", "", "");
    parameter.optopt("c", "", "", "");
    parameter.optopt("d", "", "", "");
    // optinale Parameter
    parameter.optflag("f", "", "");
    parameter.optflag("h", "", "");

    let gefunden = parameter.parse(&eingabe[1..]).unwrap();

    // Hilfe ausgeben
    if gefunden.opt_present("h") 
    {
        println!("\nPflichtparameter:");
        println!("-a <Kern ids für das Pinning> Format 1,7,3 oder 3-7>");
        println!("-b <Eingabegrößen n> Format: 10,30,100>");
        println!("-c <Anzahl Threads>");
        println!("-d <Dateiname zum Speichern der Ergebnisse>");
        println!("\noptional:");
        println!("-f (Flagge für Debugging)");
        println!("-h (Flagge für Hilfe)");
        std::process::exit(0);
    }

    // Parameter a parsen
    let a: String = gefunden.opt_str("a").unwrap_or_else(|| 
        fehlerausgabe("Parameter a wurde nicht gefunden. Benutzung siehe -h"));
    let kerne: Vec<u32> = kern_umwandeln(&a).unwrap_or_else(|_| 
        fehlerausgabe("Parameter a hat falsches Format. Benutzung siehe -h"));

    // Parameter b parsen
    let b: String = gefunden.opt_str("b").unwrap_or_else(|| 
        fehlerausgabe("Parameter b wurde nicht gefunden. Benutzung siehe -h"));
    let n: Vec<u32> = n_umwandeln(&b).unwrap_or_else(|_| 
        fehlerausgabe("Parameter b hat falsches Format. Benutzung siehe -h"));

    // Parameter c parsen
    let c: String = gefunden.opt_str("c").unwrap_or_else(|| 
        fehlerausgabe("Parameter c nicht gefunden. Benutzung siehe -h"));
    let threads: u32 = c.parse::<u32>().unwrap_or_else(|_| 
        fehlerausgabe("Parameter c hat falsches Format. Benutzung siehe -h"));

    // Parameter d parsen
    let log: String = gefunden.opt_str("d").unwrap_or_else(|| 
        fehlerausgabe("Parameter d nicht gefunden. Benutzung siehe -h"));

    // Parameter f parsen
    let flagge: bool = gefunden.opt_present("f");

        Settings { kerne, n, threads, log, flagge }
}

/*
    Gibt den Fehler aus
*/
fn fehlerausgabe(fehler: &str) -> !
{
    println!("\n{}\n", fehler);
    std::process::exit(1);
}


/*
    Wandelt einen String mit Zahlen in einen Vektor aus integer um  
*/
fn n_umwandeln(umwandeln: &str) -> Result<Vec<u32>, ()> 
{
    let mut zahlen: Vec<u32> = Vec::new();

    // Format: 1,2,3
    if umwandeln.contains(",") 
    {
        for i in umwandeln.split(',') 
        {
            let num: u32 = i.trim().parse::<u32>().map_err(|_| ())?;
            zahlen.push(num);
        }
        zahlen.sort();
        // mehrfache Zahlen entfernen
        zahlen.dedup();
        Ok(zahlen)
    }
    else 
    {
        return Err(());    
    }
}
    
/*
    Wandelt einen String mit Kern ids in einen Vektor aus integer um  
*/
fn kern_umwandeln(umwandeln: &str) -> Result<Vec<u32>, ()> 
{
    let mut zahlen: Vec<u32> = Vec::new();

    // Anzahl logischer Kerne
    let logisch: u32 = core_affinity::get_core_ids().map_or(0, |ids| ids.len() as u32);

    // Format: 1,2,3
    if umwandeln.contains(",") 
    {
        for i in umwandeln.split(',') 
        {
            let nummer: u32 = i.trim().parse::<u32>().map_err(|_| ())?;
            if nummer < logisch
            {
                zahlen.push(nummer);
            }
            else 
            {
                return Err(());  
            }
        }
        zahlen.sort();
        // mehrfache Zahlen entfernen
        zahlen.dedup(); 
    }
    else if umwandeln.contains("-")
    {
        // Format: "a-b"
        let parts: Vec<&str> = umwandeln.split('-').collect();
            
        if parts.len() != 2 
        {
            return Err(());
        }

        let a: u32 = parts[0].trim().parse::<u32>().map_err(|_| ())?;
        let b: u32 = parts[1].trim().parse::<u32>().map_err(|_| ())?;

        if a < logisch && b < logisch && b >= a
        {
            for i in a..=b 
            {
                zahlen.push(i);
            }         
        } 
        else
        {
            return Err(());
        }
    }
    else 
    {
        return Err(());    
    }

    Ok(zahlen)
}
