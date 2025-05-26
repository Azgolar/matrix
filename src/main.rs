mod verarbeiten;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::{thread, process, time::Instant, sync::Arc};
use core_affinity::{CoreId, get_core_ids, set_for_current};

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
fn multiplikation(a: Arc<Vec<Vec<u32>>>, b: Arc<Vec<Vec<u32>>>,  c: &mut Vec<Vec<u32>>, n: usize, num_threads: usize) {                     

    // Anzahl Zeilen pro Thread
    let zeilen_pro_thread: usize = n / num_threads;
    let rest: usize  = n % num_threads;

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
                let mut temporär: Vec<u32> = Vec::with_capacity(n);
                for j in 0..n {
                    let mut summe: u32 = 0;
                    // Zeile i * Spalte j
                    for k in 0..n {
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

    // Matrix zusammenbauen
    // Es werden nur die Zeiger geändert und keine Daten kopiert  -> O(1)
    for handle in handles {
        for (i, row) in handle.join().unwrap() {
            // Speichern der Zeile des zwischenergebnis in der richtigen Zeile
            c[i] = row;
        }
    }
}



fn multiply(a: Arc<Vec<Vec<u32>>>, b: Arc<Vec<Vec<u32>>>, c: &mut Vec<Vec<u32>>, n: usize, num_threads: usize) {
    // scope ist notwendig um Threads mit veränderbaren Referenzen (&mut c) zu starten, ohne dass die Referenz statisch
    // sein muss  
    thread::scope(|s| {
        // Borrow the entire output matrix C as a mutable slice of rows
        let mut ürig: &mut [Vec<u32>] = c.as_mut_slice();
        // Track the global row offset for each thread's chunk
        let mut offset: usize = 0;

        // Zeilen pro Thread
        let basis: usize = n / num_threads;
        let rest: usize  = n % num_threads;

        for z in 0..num_threads {
            // insgesamte Zeilen des Threads 
            let zeilen: usize = basis + if z < rest { 1 } else { 0 };

            // Split off the first `rows` rows from `remaining` for this chunk
            let (bearbeiten, restliche_zeilen): (&mut [Vec<u32>], &mut [Vec<u32>]) = ürig.split_at_mut(zeilen);
            // Capture the starting row index for this thread
            let anfang: usize = offset;

            // jeder Thread muss seinen eigenen Zeiger haben
            let a_zeiger: Arc<Vec<Vec<u32>>> = Arc::clone(&a);
            let b_zeiger: Arc<Vec<Vec<u32>>> = Arc::clone(&b);

            // Kern CoreId struct für pinning erzeugen 
            let kern: CoreId = CoreId { id: z };

            // Spawn the thread, moving captured variables into the closure
            s.spawn(move || {

                // Kern pinning 
                set_for_current(kern);

                // Iterate over each local row in this chunk
                for (i_lokal, ausgabe) in bearbeiten.iter_mut().enumerate() {
                    // globaler Zeilenindex
                    let i: usize = anfang + i_lokal;
                    // über Spalten iterieren
                    for j in 0..n {
                        // C[i][j] berechnen
                        let mut summe: u32 = 0;
                        for k in 0..a_zeiger[i].len() {
                            summe = summe + a_zeiger[i][k] * b_zeiger[k][j];
                        }
                        // schreiben in C
                        ausgabe[j] = summe;
                    }
                }
            }); 

            // Updaten für nächsten Thread
            ürig = restliche_zeilen;
            offset = offset + zeilen;
        }
        // wegen scoped thread ist der join automatisch
    });
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
    // Eingabe ohne Programmname
    let argument: Vec<String> = std::env::args().skip(1).collect();

    // Nutzereingabe parsen
    let (n, modus, datei, debug): (Vec<u32>, u32, String, bool) = verarbeiten::eingabe(&argument);

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
    let threads: usize = get_core_ids().map(|cores| cores.len()).unwrap_or_else(|| -> usize {
            eprintln!("Konnte logische Kerne nicht abfragen");
            process::exit(1);
        });

    // Benchmarking für alle n durchführen
    for i in 2..threads {                           
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
                multiplikation(Arc::clone(&a_teilen), Arc::clone(&b_teilen),  &mut c,aktuell, i);
            }
            else if modus == 2 {
                multiply(Arc::clone(&a_teilen), Arc::clone(&b_teilen), &mut c, aktuell, i);
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















