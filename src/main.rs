mod verarbeiten;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::{thread, time::Instant};
use std::sync::Arc;

/*
    Single Thread
*/
fn single_matrixmultiplikation(a: &Vec<Vec<u32>>, b: &Vec<Vec<u32>>, c: &mut Vec<Vec<u32>>, n: usize) {
    for i in 0..n {
        for j in 0..n {
            let mut summe: u32 = 0;
            for k in 0..n {
                summe = summe + a[i][k] * b[k][j];
            }
            c[i][j] = summe;
        }
    }
}


/*
    naive parallele Matrixmultiplikation mit manuell gestarteten Threads 
*/
fn parallel_multiplikation(
    a: &Vec<Vec<u32>>,
    b: &Vec<Vec<u32>>,
    num_threads: usize,
) -> Vec<Vec<u32>> {
    let n = a.len();                    // number of rows in A
    let p = a[0].len();                 // number of cols in A == rows in B
    let m = b[0].len();                 // number of cols in B

    // share A and B across threads
    let a_shared = Arc::new(a.clone());
    let b_shared = Arc::new(b.clone());

    // how many full rows per thread + distribute the remainder
    let base = n / num_threads;
    let rem  = n % num_threads;

    let mut handles = Vec::with_capacity(num_threads);
    for t in 0..num_threads {
        let a_clone = Arc::clone(&a_shared);
        let b_clone = Arc::clone(&b_shared);

        // compute this thread’s slice [start..end)
        let start = t * base + usize::min(t, rem);
        let end   = start + base + if t < rem { 1 } else { 0 };

        let handle = thread::spawn(move || {
            // returns Vec<(row_index, row_vec)>
            let mut out = Vec::with_capacity(end - start);
            for i in start..end {
                let mut row = Vec::with_capacity(m);
                for j in 0..m {
                    let mut sum = 0;
                    for k in 0..p {
                        sum += a_clone[i][k] * b_clone[k][j];
                    }
                    row.push(sum);
                }
                out.push((i, row));
            }
            out
        });

        handles.push(handle);
    }

    // collect and assemble
    let mut result = vec![vec![0; m]; n];
    for handle in handles {
        for (i, row) in handle.join().unwrap() {
            result[i] = row;
        }
    }

    result
}






/// Erzeugt eine n x n Matrix mit f64 Zufallswerten im Bereich [0,1[
fn zufall_matrix(n: usize, rng: &mut StdRng) -> Vec<Vec<u32>> {
    let mut matrix: Vec<Vec<u32>> = vec![vec![0; n]; n];
    for i in 0..n {
        for j in 0..n {
            matrix[i][j] = rng.random_range(0..10);
        }
    }
    matrix
}

fn main() {
    // let args: Vec<String> = std::env::args().collect();

    // Test-Einstellungen
    let test: Vec<String> = vec![
        "-n".into(), "30".into(),
        "-c".into(), "dumm".into(),
        "-d".into(),
    ];

    // Nutzereingabe parsen
    let (n, datei, debug): (Vec<u32>, String, bool) = verarbeiten::eingabe(&test);

    // Debug Eingabe
    if debug {
        println!("Einstellungen:\n-n: {:?}\n-c: {}\n", n, datei);
    }

    // Speicherplatz reserverien
    let mut laufzeit: Vec<f64> = Vec::with_capacity(n.len());

    // fester Seed
    let mut zufall: StdRng = StdRng::seed_from_u64(0xDEADBEEFCAFEBABE);

    // Threads
    //let threads: usize = get_core_ids().map(|cores| cores.len()).unwrap_or_else(|| {
      //      println!("\nKonnte logische Kerne nicht abfragen\n");
        //    process::exit(1);
        //});

    // Benchmarking für alle n durchführen
    for i in 2..6 {                           
        let aktuell: usize = n[i] as usize;

        // Zufallsmatrizen erzeugen
        let a: Vec<Vec<u32>> = zufall_matrix(aktuell, &mut zufall);
        let b: Vec<Vec<u32>> = zufall_matrix(aktuell, &mut zufall);

        // leere Ergebnismatrizen erzeugen
        let mut c_single: Vec<Vec<u32>> = vec![vec![0; aktuell]; aktuell];

        for _ in 0..n.len() {
            let start: Instant = Instant::now();

            let c_parallel = parallel_multiplikation(&a, &b, i);

            // Laufzeit in Millisekunden
            let dauer: f64 = start.elapsed().as_secs_f64() * 1000.0;
            laufzeit.push(dauer);

            // Kontrolle ausgeben
            if debug {
                single_matrixmultiplikation(&a, &b, &mut c_single, aktuell);
                vergleich(&c_single, &c_parallel);
            }
        }
        // Speichern der Ergebnisse eines Threads
        verarbeiten::speichern(&datei, &n, &laufzeit, i);
        
        // Laufzeit zurücksetzen
        laufzeit.clear();
    }
}


/*
    Vergleich der Ergebnisse von single und multithreaded
*/
fn vergleich(single: &Vec<Vec<u32>>, parallel: &Vec<Vec<u32>>) {
    for i in 0..single.len() {
        for j in 0..single[0].len() {
            if single[i][j] != parallel[i][j] {
                println!("Ergebnis falsch\n");
                return 
            }
        }
    }
    println!("Ergebnis korrekt\n");
}
