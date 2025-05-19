mod verarbeiten;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::{thread, time::Instant, sync::Arc};
use core_affinity::{CoreId, set_for_current};

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
    parallele Matrixmultiplikation mit manuell gestarteten Threads 
*/
fn multiplikation(a: Arc<Vec<Vec<u32>>>, b: Arc<Vec<Vec<u32>>>, num_threads: usize) -> Vec<Vec<u32>> {
    
    // Anzahl Zeilen
    let anzahl_zeilen: usize = a.len();                    
    // Anzahl spalten
    let anzahl_spalten: usize = a[0].len();                                 

    // Anzahl Zeilen pro Thread
    let zeilen_pro_thread: usize = anzahl_zeilen / num_threads;
    let rest: usize  = anzahl_zeilen % num_threads;

    // für schließen der Threads
    let mut handles: Vec<thread::JoinHandle<Vec<(usize, Vec<u32>)>>> = Vec::with_capacity(num_threads);

    // benötigte Anzahl Threads
    for z in 0..num_threads  {

        // Kopie des Ark Zeigers für jeden Thread erzeugen
        let a_zeiger: Arc<Vec<Vec<u32>>> = Arc::clone(&a);
        let b_zeiger: Arc<Vec<Vec<u32>>> = Arc::clone(&b);

        // struct für Thread pinning
        let kern: CoreId = CoreId { id: z };

        // berechnet den Bereich [anfang,ende[ für jeden Thread
        let anfang: usize = z * zeilen_pro_thread + usize::min(z, rest);
        let ende: usize   = anfang + zeilen_pro_thread + if z < rest { 1 } else { 0 };

        // Threads erzeugen
        let erzeugen: thread::JoinHandle<Vec<(usize, Vec<u32>)>> = thread::spawn(move || {

            // Kern auf logischen Prozessorkern pinnen
            set_for_current(kern);

            // speichert die zeilen für jeden Thread
            // jedes Element ist ein Tupel aus (zeilenindex, Zeile)
            let mut zwischenergebnis: Vec<(usize, Vec<u32>)> = Vec::with_capacity(ende - anfang);
            for i in anfang..ende {
                // speichert die berechnet zeile
                let mut temporär: Vec<u32> = Vec::with_capacity(anzahl_spalten);
                for j in 0..anzahl_spalten {
                    let mut summe: u32 = 0;
                    // Zeile i * Spalte j
                    for k in 0..anzahl_spalten {
                        summe = summe + a_zeiger[i][k] * b_zeiger[k][j];
                    }
                    temporär.push(summe);
                }
                zwischenergebnis.push((i, temporär));
            }
            // Vektor mit Tuplen aus (Zeilenindex, Zeile) zurückgeben
            zwischenergebnis
        });

        // Thread handle für join speichern
        handles.push(erzeugen);
    }

    // Ergebnismatrix mit null initilaisieren
    let mut ergebnis: Vec<Vec<u32>> = vec![Vec::with_capacity(anzahl_spalten); anzahl_zeilen];
    // Matrix zusammenbauen
    // Es werden nur die Zeiger geändert und keine Daten kopiert  -> O(1)
    for handle in handles {
        for (i, row) in handle.join().unwrap() {
            // Speichern der Zeile des zwischenergebnis in der richtigen Zeile
            ergebnis[i] = row;
        }
    }

    ergebnis
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
        "-b".into(), "1".into(),
        "-c".into(), "ergebnis".into(),
        "-d".into(),
    ];

    // Nutzereingabe parsen
    let (n, modus, datei, debug): (Vec<u32>, u32, String, bool) = verarbeiten::eingabe(&test);

    // Debug Eingabe
    if debug {
        let s: &'static str = if modus == 1 {
            "regulär parallel"
        } else if modus == 2 {
            "loop unrolling"
        } else if modus == 3 {
            "block tiling"
        } else if modus == 4 {
            "rayon"
        } else {
            "crossbeam"
        };
        println!("Einstellungen:\n-n: {:?}\n-b: {}\n-c: {}\n", n, s, datei);
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

        // 2) wrap them in Arcs once, before benchmarking loop
        let a_teilen: Arc<Vec<Vec<u32>>> = Arc::new(a);
        let b_teilen: Arc<Vec<Vec<u32>>> = Arc::new(b);

        // leere Ergebnismatrizen erzeugen
        let mut c_single: Vec<Vec<u32>> = vec![vec![0; aktuell]; aktuell];
        let mut c: Vec<Vec<u32>> = vec![vec![0; aktuell]; aktuell];

        for _ in 0..n.len() {
            let start: Instant = Instant::now();

            if modus == 1 {
                c = multiplikation(Arc::clone(&a_teilen), Arc::clone(&b_teilen), i);
            }

            // Laufzeit in Millisekunden
            let dauer: f64 = start.elapsed().as_secs_f64() * 1000.0;
            laufzeit.push(dauer);

            // Kontrolle ausgeben
            if debug {
                single_matrixmultiplikation(&*a_teilen, &*b_teilen, &mut c_single, aktuell);
                vergleich(&c_single, &c);
            }
        }
        // Speichern der Ergebnisse eines Threads
        verarbeiten::speichern(&datei, &n, &laufzeit, i);

        // Laufzeit zurücksetzen
        laufzeit.clear();

        println!("Benchmark Thread {} beendet", i);
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
















fn multiply(
    a: Arc<Vec<Vec<u32>>>,
    b: Arc<Vec<Vec<u32>>>,
    num_threads: usize,
) -> Vec<Vec<u32>> {
    let n = a.len();
    let m = b[0].len();

    // Allocate the full result matrix here
    let mut result = vec![vec![0; m]; n];

    thread::scope(|s| {
        // Borrow the whole matrix one time
        let mut remaining: &mut [Vec<u32>] = result.as_mut_slice();
        let mut row_offset = 0;

        // How many rows per thread
        let base = n / num_threads;
        let rem  = n % num_threads;

        for t in 0..num_threads {
            // Compute slice size for this thread
            let rows = base + if t < rem { 1 } else { 0 };

            // Split off the front `rows` from `remaining`
            let (chunk, tail) = remaining.split_at_mut(rows);

            // Capture the start index so your closure knows the global row
            let offset = row_offset;
            let a_cloned = Arc::clone(&a);
            let b_cloned = Arc::clone(&b);

            // Spawn the thread, moving in *only* this chunk (disjoint &mut)
            s.spawn(move || {
                for (i_local, row_out) in chunk.iter_mut().enumerate() {
                    let i = offset + i_local;
                    for j in 0..m {
                        let mut sum = 0;
                        for k in 0..a_cloned[i].len() {
                            sum += a_cloned[i][k] * b_cloned[k][j];
                        }
                        row_out[j] = sum;
                    }
                }
            });

            // Prepare for next iteration
            remaining = tail;
            row_offset += rows;
        }
    }); // ← threads are all joined here

    result
}
