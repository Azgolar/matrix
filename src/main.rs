mod log;
mod eingabe;
use std::time::Instant;
use rand::{Rng, SeedableRng, rngs::StdRng};

fn matrixmultiplikation(a: &Vec<Vec<f64>>, b: &Vec<Vec<f64>>, n: u32, threads: u32)
{

}



/// Erzeugt eine n x n Matrix mit f64 Zufallswerten im Bereich [0,1[
fn zufall_matrix(n: usize, rng: &mut StdRng) -> Vec<Vec<f64>> 
{
    let mut matrix: Vec<Vec<f64>> = vec![vec![0.0; n]; n];
    for i in 0..n 
    {
        for j in 0..n 
        {
            matrix[i][j] = rng.random::<f64>();
        }
    }
    matrix
}




fn main() 
{
    //let gefunden = parameter.parse(&env::args()
          //  .skip(1).collect::<Vec<_>>()).unwrap_or_else(|e| 
           // { Einstellungen::fehlerausgabe(&format!("Fehler beim Parsen des Arguments: {}", e))});


        // Test-Einstellungen
    let test: Vec<String> = vec![
        "-a".into(), "12-18".into(),
        "-b".into(), "4,5,6".into(),
        "-c".into(), "4".into(),
        "-d".into(), "logfile.txt".into(),
        "-f".into(),
    ];

    // Nutzereingabe parsen
    let einstellungen: eingabe::Settings = eingabe::verarbeiten(&test);

    // Debug Ausgabe
    if einstellungen.flagge
    {
        println!("Einstellungen: {:#?}", einstellungen);
    }

    // Speicherplatz reserverien
    let mut laufzeit: Vec<f64> = Vec::with_capacity(einstellungen.n.len());

    // fester Seed
    let mut zufall: StdRng = StdRng::seed_from_u64(0xDEADBEEFCAFEBABE);

    // Benchmarking für all n durchführen
    for i in 0..einstellungen.n.len()
    {
        // Zwei Zufallsmatrizen erzeugen
        let a: Vec<Vec<f64>> = zufall_matrix(einstellungen.n[i] as usize, &mut zufall);
        let b: Vec<Vec<f64>> = zufall_matrix(einstellungen.n[i] as usize, &mut zufall);


        let anfang: Instant = Instant::now();
        
        matrixmultiplikation(&a, &b, einstellungen.n[i], einstellungen.threads);

        // Laufzeit in Millisekunden
        let dauer:f64  = anfang.elapsed().as_secs_f64() * 1000.0;
        laufzeit.push(dauer);
    }

    // Speichern der ergebnisse
    log::speichern(&einstellungen, &laufzeit);
}


