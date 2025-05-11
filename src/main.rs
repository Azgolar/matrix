mod log;
mod eingabe;

fn main() 
{
    //let gefunden = parameter.parse(&env::args()
          //  .skip(1).collect::<Vec<_>>()).unwrap_or_else(|e| 
           // { Einstellungen::fehlerausgabe(&format!("Fehler beim Parsen des Arguments: {}", e))});

    // Speichern der ergebnisse
    log::speichern("test.txt", "5","10,20,30", "1.5,2.5,3.5");
}
