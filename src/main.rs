mod verarbeiten;
use std::{time::Instant, thread, process};
use rand::{Rng, SeedableRng, rngs::StdRng};
use core_affinity::get_core_ids;

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
    // let args: Vec<String> = std::env::args().collect();

    // Test-Einstellungen
    let test: Vec<String> = vec![
        "-n".into(), "4-6".into(),
        "-c".into(), "ergebnis.txt".into(),
        "-d".into(),
    ];

    // Nutzereingabe parsen
    let (n, datei, debug): (Vec<u32>, String, bool) = verarbeiten::verarbeiten(&test)
        .unwrap_or_else(|fehler| {
                println!("\n{}\n", fehler);
                process::exit(1);
            });

    // struct Debug Ausgabe
    if debug
    {
        println!("Einstellungen:\n-n: {:?}\n-c: {}\n", n, datei);
    }

    // Speicherplatz reserverien
    let mut laufzeit: Vec<f64> = Vec::with_capacity(n.len());

    // fester Seed
    let mut zufall: StdRng = StdRng::seed_from_u64(0xDEADBEEFCAFEBABE);

    // Threads
    let threads: usize = get_core_ids().map(|cores| cores.len()).unwrap_or_else(|| {
            println!("\nKonnte logische Kerne nicht abfragen\n");
            process::exit(1);
        });

    // Benchmarking für alle n durchführen
    for i in 0..n.len()
    {
        let n: usize = n[i] as usize;

        // Zwei Zufallsmatrizen erzeugen
        let a: Vec<Vec<f64>> = zufall_matrix(n, &mut zufall);
        let b: Vec<Vec<f64>> = zufall_matrix(n, &mut zufall);

        // Ergebnismatrix initialisieren
        let mut single: Vec<Vec<f64>> = vec![vec![0.0; n]; n];
        let mut parallel: Vec<Vec<f64>> = vec![vec![0.0; n]; n];

        let anfang: Instant = Instant::now();
        
        parallel_matrixmultiplikation(&a, &b, &mut parallel, n, threads);

        // Laufzeit in Millisekunden
        let dauer:f64  = anfang.elapsed().as_secs_f64() * 1000.0;
        laufzeit.push(dauer);

        // Kontrolle ausgeben
        if debug
        {
            single_matrixmultiplikation(&a, &b, &mut single, n);
            vergleich(&single, &parallel, n);
        }
    }

    // Speichern der Ergebnisse
    verarbeiten::speichern(&datei, &laufzeit, threads);
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
