mod log;
mod eingabe;
use std::{time::Instant, thread};
use rand::{Rng, SeedableRng, rngs::StdRng};

/*
    Single Thread
*/
fn single_matrixmultiplikation(a: &Vec<Vec<f64>>, b: &Vec<Vec<f64>>, c: &mut Vec<Vec<f64>>, n: usize) 
{
    for i in 0..n 
    {
        for j in 0..n 
        {
            let mut summe: f64 = 0.0;
            for k in 0..n 
            {
                summe = summe + a[i][k] * b[k][j];
            }
            c[i][j] = summe;
        }
    }
}

fn parallel_matrixmultiplikation(a: &Vec<Vec<f64>>, b: &Vec<Vec<f64>>, c: &mut Vec<Vec<f64>>, n: usize, mut threads: usize)
{
    // maximal so viele Threads wie Zeilen. 
    // threads = 0 wurde bei der eingabe bereits geprüft
    if threads > n
    {
        threads = n;
    }

    // Zeilen pro Thread
    let anzahl_zeilen: usize = n / threads;
    let rest: usize = n % threads;

    /*
        Threads starten
        Threads sind als closure in Rust implementiert und dürfen so lokale Variablen nutzen 
    */
    thread::scope(|s| 
        {
            // parallelisierung
        });
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

    // struct Debug Ausgabe
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
        let n = einstellungen.n[i] as usize;

        // Zwei Zufallsmatrizen erzeugen
        let a: Vec<Vec<f64>> = zufall_matrix(n, &mut zufall);
        let b: Vec<Vec<f64>> = zufall_matrix(n, &mut zufall);
        // Ergebnismatrix initialisieren
        let mut single: Vec<Vec<f64>> = vec![vec![0.0; n]; n];
        let mut parallel: Vec<Vec<f64>> = vec![vec![0.0; n]; n];

        let anfang: Instant = Instant::now();
        
        parallel_matrixmultiplikation(&a, &b, &mut parallel, n, einstellungen.threads as usize);


        // Laufzeit in Millisekunden
        let dauer:f64  = anfang.elapsed().as_secs_f64() * 1000.0;
        laufzeit.push(dauer);

        // Kontrolle ausgeben
        if einstellungen.flagge
        {
            single_matrixmultiplikation(&a, &b, &mut single, n);
            vergleich(&single, &parallel, n);
        }
    }

    // Speichern der ergebnisse
    log::speichern(&einstellungen, &laufzeit);
}



/*
    Vergleich der Ergebnisse von single und multithreaded
*/
fn vergleich(single: &Vec<Vec<f64>>, parallel: &Vec<Vec<f64>>, n: usize) 
{
    if single == parallel {
        println!("\nErgebnisse stimmen überein\n");
    } else {
        eprintln!("\nErgebnisse weichen ab\n");
        'outer: for i in 0..n 
        {
            for j in 0..n 
            {
                if (single[i][j] - parallel[i][j]).abs() > 1e-12 
                {
                    println!("Erste Abweichung bei ({}, {}): single={} vs parallel={}",
                        i, j, single[i][j], parallel[i][j],);
                    break 'outer;
                }
            }
        }
    }
}
