# Rust cheat sheet — ScreenRecorder

Poznámky k tomu, čo sme si prešli pri učení Rustu na tomto projekte. Príklady sú učebné úryvky, nie hotová implementácia recorderu. Dokument budeme dopĺňať pri ďalších krokoch.

## Cargo

```powershell
cargo run                  # Spustí hlavnú aplikáciu
cargo run --example buffer # Spustí examples/buffer.rs
cargo check                # Skontroluje kód bez vytvorenia výsledného executable
cargo fmt                  # Naformátuje Rust kód
```

`Cargo.toml` obsahuje nastavenia balíka a závislosti. `config.toml` je naša vlastná konfigurácia aplikácie.

## Premenné a základné typy

```rust
let fps: u32 = 60;          // Nemenná premenná
let mut count: usize = 0;   // Premennú môžeme meniť
count += 1;
```

| Rust | Význam / približná analógia v C++ |
|---|---|
| `u8` | Nezáporné 8-bitové číslo; `uint8_t` |
| `u32` | Nezáporné 32-bitové číslo; `uint32_t` |
| `u64` | Nezáporné 64-bitové číslo; `uint64_t` |
| `usize` | Typ pre indexy a veľkosti kolekcií; podobný `size_t` |
| `String` | Vlastnený, meniteľný UTF-8 text |
| `&str` | Požičaný pohľad na UTF-8 text |
| `Vec<T>` | Dynamické pole; podobné `std::vector<T>` |
| `PathBuf` | Vlastnená cesta v súborovom systéme |
| `()` | Jednotkový typ: žiadna užitočná návratová hodnota |

## Štruktúry, enumy a metódy

```rust
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct RecordingConfig {
    output_directory: PathBuf,
    format: VideoFormat,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum VideoFormat {
    Mp4,
    Mkv,
}
```

- `struct` zoskupuje polia; `enum` definuje možné varianty hodnoty.
- `derive(Debug)` vygeneruje podporu debug výpisu.
- `derive(Deserialize)` vygeneruje podporu načítania dát cez Serde.
- `rename_all = "lowercase"` mapuje napríklad `Mp4` na `"mp4"` v TOML.
- `pub` sprístupní položku navonok v rámci pravidiel viditeľnosti modulov. Polia štruktúry majú vlastnú viditeľnosť.
- Povolený variant konfigurácie ešte neznamená implementovanú podporu formátu videa.

```rust
impl RecordingConfig {
    fn validate(&self) -> Result<(), String> {
        if self.output_directory.as_os_str().is_empty() {
            return Err("Cesta nesmie byť prázdna".to_string());
        }

        Ok(())
    }
}
```

`impl` pridáva metódy typu. `&self` si existujúcu hodnotu požičiava na čítanie. Posledný výraz bez bodkočiarky je návratová hodnota; `return` umožňuje skončiť skôr.

## Vytvorenie štruktúry s pomenovanými poľami

Naša štruktúra `VideoBlock` má pomenované polia, preto hodnotu vytvárame cez zložené zátvorky:

```rust
fn block(id: u32, duration: Duration) -> VideoBlock {
    VideoBlock {
        id,
        data: vec![42u8; 4],
        duration,
    }
}
```

`id` je skratka za `id: id` a `duration` za `duration: duration`. Výraz bez záverečnej bodkočiarky sa vráti z funkcie. `VideoBlock(...)` nie je konštruktor našej pomenovanej štruktúry: okrúhle zátvorky používame pre volanie funkcie alebo konštrukciu tuple struct, napríklad `struct Point(u32, u32);` a `Point(1, 2)`.

Pozor na rozdiel medzi `vec![42u8; 4]` (štyri bajty `[42, 42, 42, 42]`) a `vec![42u8, 4]` (dva bajty `[42, 4]`). Ak pomocnú funkciu so štyrmi bajtmi použijeme pre tri bloky v `main`, výsledok bude 12 bajtov, nie pôvodných 3 072.

## Makrá a výpis

Výkričník v `println!` označuje makro. Makro sa pri kompilácii rozvinie na Rust kód.

```rust
println!("FPS: {}", 60);      // Bežné formátovanie cez Display
println!("{value:?}");       // Debug výpis
println!("{value:#?}");      // Debug výpis s odsadením
eprintln!("Chyba: {error}"); // Chybový výstup
let text = format!("FPS: {}", 60); // Vytvorí String bez vypisovania
let bytes = vec![0u8; 1024];       // Vec s 1 024 nulovými bajtmi
```

`println!` pridáva nový riadok. `:?` potrebuje trait `Debug`, samotné `{}` používa `Display`. Trait definuje spoločné správanie typov.

## Result, Option a operátor ?

| Typ | Úspech / prítomnosť | Chyba / neprítomnosť |
|---|---|---|
| `Result<T, E>` | `Ok(value)` | `Err(error)` |
| `Option<T>` | `Some(value)` | `None` |

```rust
use std::{error::Error, fs};

fn read_config_text() -> Result<String, Box<dyn Error>> {
    let text = fs::read_to_string("config.toml")?;
    Ok(text)
}
```

`?` z výsledku `Ok` vytiahne hodnotu. Pri `Err` skončí funkciu a odovzdá chybu volajúcemu, prípadne ju prevedie na návratový typ chyby. Nevyvoláva paniku ako `expect` pri chybe.

`Box<dyn Error>` vlastní chybu na halde a umožňuje vracať rôzne konkrétne typy implementujúce `Error`. V našom loaderi dokáže prijať aj text chyby z `Result<(), String>`.

```rust
match read_config_text() {
    Ok(text) => println!("{text}"),
    Err(error) => eprintln!("Chyba: {error}"),
}
```

`match` spracúva varianty a musí pokryť všetky možnosti. Ak nás zaujíma iba jeden variant, môžeme použiť `if let`:

```rust
if let Some(removed) = buffer.pop_front() {
    println!("Odstránený prvok: {removed:?}");
}
```

`ExitCode::SUCCESS` a `ExitCode::FAILURE` z `std::process` umožňujú funkcii `main` oznámiť systému výsledok spustenia.

## Ownership: presun, kopírovanie a požičanie

Pýtaj sa: **Odovzdávam vlastníctvo, alebo iba požičiavam prístup?**

| Parameter | Správanie |
|---|---|
| `block: VideoBlock` | Prijíma hodnotu; pri našom ne-`Copy` type preberá vlastníctvo |
| `block: &VideoBlock` | Požičiava si zdieľaný prístup na čítanie |
| `block: &mut VideoBlock` | Požičiava si výhradný prístup umožňujúci zmeny |

Počas aktívneho výhradného požičania nemôžeme súčasne používať iné referencie na tú istú hodnotu. Rust kontroluje aj to, aby referencia neprežila dáta, na ktoré ukazuje.

```rust
let a = 42u32;
let b = a;           // u32 je Copy: hodnota sa skopíruje
println!("{a} {b}"); // Obe premenné zostávajú použiteľné

let first = vec![0u8; 1024];
let second = first;  // Vec nie je Copy: vlastníctvo sa presunie
// first už nemôžeme používať; bajty sa týmto presunom nekopírujú
```

Presun je podobný C++ move semantike, ale Rust nedovolí čítať presunutú hodnotu. `clone()` pri `Vec<u8>` vytvorí samostatnú kópiu bajtov; klonovanie teda nie je všeobecne lacné.

Keď vlastník zanikne, jeho zdroje sa automaticky uvoľnia. Je to podobné C++ RAII, bez ručného `delete` a bez trasovacieho garbage collectora.

## Posledné použitie, koniec požičania a drop

Lokálna vlastnená hodnota ako `Vec` sa bežne zruší na konci svojho bloku, nie automaticky pri poslednom použití. Ak vznikne vo funkcii a naposledy ju použijeme na riadku 50, môže držať pamäť až do konca funkcie na riadku 100.

```rust
let snapshot = vec![0u8; 1024];
println!("{}", snapshot.len()); // Posledné použitie samo osebe nespustí drop
drop(snapshot);                // Tu explicitne zrušíme vlastnenú hodnotu
// Ďalšia práca už snapshot nedrží.
```

Alternatíva je užší blok `{ ... }`: hodnota sa zruší na jeho konci. Pri presune hodnoty do inej premennej alebo vlákna preberá zodpovednosť nový vlastník.

**Koniec požičania je iná vec.** Borrow checker vie ukončiť požičanie po poslednom použití referencie, aj keď blok pokračuje. Toto správanie sa označuje ako non-lexical lifetimes (NLL).

```rust
let mut values = vec![1, 2, 3];
let borrowed = &values;
println!("{borrowed:?}"); // Posledné použitie referencie
values.push(4);           // Povolené: zdieľané požičanie už nie je potrebné
// Samotný values stále existuje.
```

`drop(&snapshot)` odovzdá iba referenciu a nezruší vlastnený snapshot. Použi `drop(snapshot)`. Pri `Vec<Arc<VideoBlock>>` sa tým zrušia odkazy snapshotu; video dáta sa uvoľnia až po zániku posledného silného vlastníka každého bloku. Uvoľnenie alokácie pritom nemusí okamžite znížiť pamäť procesu zobrazovanú Windows: alokátor môže pamäť ponechať na ďalšie použitie.

## VecDeque: rotačný buffer

`VecDeque<T>` umožňuje efektívne pridávať a odoberať na oboch koncoch. Používa kruhový buffer: logické poradie môže byť v alokácii rozdelené na dve časti. API sa podobá `std::deque`, implementácia je odlišná.

```rust
use std::collections::VecDeque;

let capacity: usize = 5; // Tento príklad vyžaduje kladnú kapacitu
let mut buffer: VecDeque<u32> = VecDeque::with_capacity(capacity);

for block in 1..=8 {
    if buffer.len() == capacity {
        buffer.pop_front();
    }
    buffer.push_back(block);
}

// Obsah: [4, 5, 6, 7, 8]
```

- `with_capacity(n)` rezervuje priestor aspoň pre `n` prvkov, ale dĺžka zostane 0. Podobá sa C++ `reserve(n)`.
- Rezervovaná kapacita nie je pevný limit. Limit vynucuje náš kód.
- Najskôr odoberáme, potom pridávame, aby sme nepotrebovali extra slot.
- Pri dostatočnej kapacite sú tieto operácie O(1); ostatné prvky sa neposúvajú.
- Priestor pre položky kolekcie a pamäť vnorených `Vec<u8>` sú samostatné alokácie.
- `1..=8` zahŕňa aj 8; `1..8` končí na 7.

Tento príklad obmedzuje počet prvkov. Recorder bude potrebovať časový limit a rešpektovanie kľúčových snímok videa.

## Iterácia a closures

```rust
for block in &buffer {
    // Iba požičiavame prvky; zostávajú v bufferi.
}
```

`for block in buffer` naopak spotrebuje kolekciu a odovzdáva jej prvky hodnotou.

```rust
let snapshot: Vec<Arc<VideoBlock>> = buffer
    .iter()
    .map(|block| Arc::clone(block))
    .collect();
```

- `.iter()` vytvorí iterátor cez zdieľané referencie.
- `|block| ...` je closure, podobná C++ lambde.
- `.map(...)` určí transformáciu každého prvku.
- `.collect()` prejde iterátor a zhromaždí výsledky do cieľovej kolekcie.

Tento úryvok predpokladá buffer typu `VecDeque<Arc<VideoBlock>>` a import `Arc`.

## Arc: zdieľané vlastníctvo bez kopírovania videa

```rust
use std::sync::Arc;

struct VideoBlock {
    id: u32,
    data: Vec<u8>,
}

let block = Arc::new(VideoBlock {
    id: 1,
    data: vec![0u8; 1024],
});

let saving = Arc::clone(&block);
assert_eq!(Arc::strong_count(&block), 2);

drop(block); // Zrušíme jedného vlastníka; saving stále drží dáta
println!("{}", saving.data.len());
```

- `Arc<T>` je podobný C++ `std::shared_ptr<T>` a používa atómové počítanie vlastníkov.
- `Arc::clone` vytvorí ďalší vlastniaci odkaz. Nekopíruje vnútorný `VideoBlock` ani jeho bajty.
- Hodnota sa zruší, keď zanikne posledný silný vlastník.
- `drop(value)` hodnotu spotrebuje a zruší teraz namiesto na konci jej rozsahu.
- `strong_count` používame na ukážku počtu vlastníkov, nie na synchronizáciu medzi vláknami.
- `Arc` sám neposkytuje možnosť meniť zdieľané dáta a neurobí ľubovoľný vnútorný typ bezpečným pre vlákna. Naše hotové video bloky budú nemenné.

Snapshot buffera vytvorí novú kolekciu odkazov, nie druhú kópiu video dát. Ak buffer starý blok odstráni, snapshot ho stále drží. Počas ukladania preto môžu v RAM zostať staršie bloky popri nových.

## Pracovné vlákno: spawn, move a join

Nasledujúci samostatný príklad simuluje pomalé spracovanie snapshotu. Nevytvára video ani nič nezapisuje na disk. Môžeš ho vložiť do `examples/buffer.rs` a spustiť cez `cargo run --example buffer`.

```rust
use std::{collections::VecDeque, sync::Arc, thread, time::Duration};

struct VideoBlock {
    id: u32,
    data: Vec<u8>,
}

fn main() {
    let mut buffer: VecDeque<Arc<VideoBlock>> = VecDeque::new();

    for id in 1..=3 {
        buffer.push_back(Arc::new(VideoBlock {
            id,
            data: vec![0u8; 1024],
        }));
    }

    let snapshot: Vec<Arc<VideoBlock>> =
        buffer.iter().map(Arc::clone).collect();

    let worker = thread::spawn(move || {
        for block in &snapshot {
            println!("Pracovné vlákno: blok {}, {} bajtov", block.id, block.data.len());
            thread::sleep(Duration::from_millis(200));
        }
        println!("Pracovné vlákno: hotovo");
    });

    buffer.clear();
    buffer.push_back(Arc::new(VideoBlock {
        id: 4,
        data: vec![0u8; 1024],
    }));
    println!("Hlavné vlákno: buffer teraz obsahuje nový blok 4");

    worker.join().expect("Pracovné vlákno skončilo panikou");
    println!("Hlavné vlákno: pracovné vlákno skončilo");
}
```

- `thread::spawn` spustí closure v novom vlákne a vráti `JoinHandle`.
- `move || { ... }` je closure bez argumentov, ktorá zachytené hodnoty prevezme hodnotou. Tu presunie snapshot do closure; video bajty nekopíruje.
- Pôvodné vlákno už snapshot nemôže používať. Buffer mu však zostáva, pretože ho closure nezachytáva.
- `clear()` odstráni všetky položky buffera. Bloky 1–3 počas práce drží snapshot; ak už práca skončila, môžu sa uvoľniť pri odstránení posledných odkazov z buffera.
- `sleep` uspí iba vlákno, ktoré ho zavolalo. Tu predstavuje pomalú prácu, nie synchronizáciu ani reálne ukladanie.
- Poradie výpisov medzi vláknami nie je garantované. Výpis za úspešným `join()` však príde až po skončení pracovníka.
- `join()` čaká na skončenie vlákna a spotrebuje jeho handle. `Err` v tomto príklade znamená paniku pracovníka; `expect` používame iba ako skratku v učebnom príklade.
- Ukončenie `main` nečaká automaticky na ostatné vlákna. Samotné zahodenie `JoinHandle` tiež nečaká; vlákno sa odpojí.
- V tray aplikácii nebudeme čakať cez `join()` pri každom stlačení skratky v obsluhe UI. Dokončenie oznámime správou; riadené čakanie vyriešime pri ukončovaní.

Tu netreba `Mutex`: každé vlákno vlastní svoju kolekciu a spoločné video bloky iba číta. `Arc` neslúži na súbežné mutovanie samotného buffera.

## Kanál medzi vláknami: mpsc

Pracovník môže hlavnému vláknu poslať správu namiesto toho, aby hlavné vlákno okamžite čakalo cez `join()`.

```rust
use std::sync::mpsc;

let (sender, receiver) = mpsc::channel::<usize>();
```

Kanál má odosielateľa (`Sender`) a príjemcu (`Receiver`). `usize` je typ správy, v našej ukážke počet spracovaných bajtov. Zápis `(sender, receiver)` rozbalí dvojicu hodnôt. `::<usize>` explicitne určuje generický parameter volanej funkcie. Skratka mpsc znamená multiple producers, single consumer: odosielateľov môže byť viac, príjemca je jeden.

Nasledujúci úryvok nadväzuje na príklad s vláknom. Nechaj vytvorenie buffera a snapshotu, pridaj import `mpsc` a nahraď kód od `let worker` po koniec tela `main`:

```rust
let (sender, receiver) = mpsc::channel::<usize>();

let worker = thread::spawn(move || {
    let mut total_bytes = 0;

    for block in &snapshot {
        total_bytes += block.data.len();
        thread::sleep(Duration::from_millis(200));
    }

    drop(snapshot);

    if sender.send(total_bytes).is_err() {
        eprintln!("Príjemca správy už neexistuje");
    }
});

let mut next_id = 4;

loop {
    match receiver.try_recv() {
        Ok(bytes) => {
            println!("Pracovník spracoval {bytes} bajtov");
            break;
        }
        Err(mpsc::TryRecvError::Empty) => {}
        Err(mpsc::TryRecvError::Disconnected) => {
            eprintln!("Pracovník neposlal výsledok a kanál sa odpojil");
            break;
        }
    }

    // Simulácia ďalšieho nahrávania, kapacita sú 3 bloky.
    if buffer.len() == 3 {
        buffer.pop_front();
    }
    buffer.push_back(Arc::new(VideoBlock {
        id: next_id,
        data: vec![0u8; 1024],
    }));
    println!("Buffer prijal nový blok {next_id}");
    next_id += 1;

    thread::sleep(Duration::from_millis(100));
}

worker.join().expect("Pracovné vlákno skončilo panikou");
```

- `sender.send(value)` odovzdá správu kanálu. Úspech znamená odovzdanie kanálu, nie potvrdenie spracovania príjemcom. Chyba znamená, že príjemca už zanikol.
- `try_recv()` nečaká: vráti správu, `Empty` (zatiaľ nič) alebo `Disconnected` (žiadna ďalšia správa už nepríde).
- `recv()` naopak čaká, kým je správa dostupná alebo sa kanál odpojí.
- `move` tu prenesie do pracovníka snapshot aj sender. Receiver a buffer zostávajú v hlavnom vlákne.
- `loop` opakuje telo; `break` ukončí cyklus.
- `sleep` v hlavnom vlákne iba spomaľuje simuláciu a bráni nepretržitému vyťažovaniu CPU. Nie je to návrh event loopu tray aplikácie.
- Výsledok tejto ukážky je 3 072 spracovaných bajtov. Počet nových blokov závisí od plánovania vlákien.
- `join()` voláme až pri ukončení príkladu. Doručenie správy ešte samo osebe nezaručuje, že pracovník už skončil; `join()` to potvrdí.
- Stále nespracúvame reálne video ani nezapisujeme na disk. V aplikácii bude správa rozlišovať úspešné uloženie a chybu. Ukážka po výsledku skončí, skutočný recorder bude pokračovať.

## Výsledok práce verzus výsledok prijatia správy

Kanál môže prenášať aj `Result`. Tak rozlíšime chybu samotnej práce od odpojenia kanála. Nasledujúci krok stále iba simuluje spracovanie bajtov, nie zápis videa.

Do `examples/buffer.rs` nad `main` pridaj:

```rust
fn process_snapshot(snapshot: Vec<Arc<VideoBlock>>) -> Result<usize, String> {
    if snapshot.is_empty() {
        return Err("Snapshot je prázdny".to_string());
    }

    let mut total_bytes = 0;

    for block in &snapshot {
        if block.data.is_empty() {
            return Err(format!("Blok {} nemá dáta", block.id));
        }

        total_bytes += block.data.len();
        thread::sleep(Duration::from_millis(200));
    }

    Ok(total_bytes)
}
```

Funkcia prevezme vlastníctvo snapshotu. Pri návrate, vrátane skorého `return Err(...)`, sa snapshot automaticky zruší. `Arc` odkazy buffera tým nezaniknú. Explicitný `drop(snapshot)` tu nepotrebujeme.

V `main` nahraď vytvorenie kanála a pracovníka:

```rust
let (sender, receiver) = mpsc::channel::<Result<usize, String>>();

let worker = thread::spawn(move || {
    let result = process_snapshot(snapshot);

    if sender.send(result).is_err() {
        eprintln!("Príjemca správy už neexistuje");
    }
});
```

Teraz `try_recv()` vracia `Result<Result<usize, String>, mpsc::TryRecvError>`. Namiesto pôvodnej vetvy `Ok(bytes)` použij dve vetvy; existujúce `Empty` a `Disconnected` nechaj:

```rust
Ok(Ok(bytes)) => {
    println!("Spracovanie dokončené: {bytes} bajtov");
    break;
}
Ok(Err(error)) => {
    eprintln!("Spracovanie zlyhalo: {error}");
    break;
}
```

| Výsledok try_recv | Význam |
|---|---|
| `Ok(Ok(bytes))` | Správa prišla a práca uspela |
| `Ok(Err(error))` | Správa prišla a práca zlyhala očakávanou chybou |
| `Err(Empty)` | Zatiaľ neprišla správa |
| `Err(Disconnected)` | Kanál je prázdny a už nemá odosielateľa |

Vnútorné `Err` nie je panika vlákna. Pracovník môže korektne odoslať chybu a skončiť; `join()` potom uspeje. Jeho `Err` označuje paniku pri unwindingu, nie neúspech práce v prenesenej správe.

Ručné overenie: bežné tri bloky po 1 024 bajtov vrátia 3 072. Pri vytváraní pôvodných blokov skús dočasne `data: if id == 2 { Vec::new() } else { vec![0u8; 1024] }`. Pracovník potom pošle chybu `Blok 2 nemá dáta`. Po overení vráť pôvodné dáta. Ide o učebnú validáciu, nie definíciu všetkých pravidiel reálneho video formátu.

## Vlastný typ ReplayBuffer: struct a impl

Pravidlá obmedzenia počtu blokov a vytváranie snapshotu presunieme do jedného typu. Nasledujúci učebný typ stále obmedzuje počet blokov, nie dĺžku videa. Zatiaľ ho vlož pod `VideoBlock` v `examples/buffer.rs`.

```rust
struct ReplayBuffer {
    blocks: VecDeque<Arc<VideoBlock>>,
    max_blocks: usize,
}

impl ReplayBuffer {
    fn new(max_blocks: usize) -> Result<Self, String> {
        if max_blocks == 0 {
            return Err("Buffer musí mať miesto aspoň pre jeden blok".to_string());
        }

        Ok(Self {
            blocks: VecDeque::with_capacity(max_blocks),
            max_blocks,
        })
    }

    fn push(&mut self, block: VideoBlock) {
        if self.blocks.len() == self.max_blocks {
            self.blocks.pop_front();
        }

        self.blocks.push_back(Arc::new(block));
    }

    fn snapshot(&self) -> Vec<Arc<VideoBlock>> {
        self.blocks.iter().map(Arc::clone).collect()
    }
}
```

- `Self` v tomto `impl` znamená `ReplayBuffer`; `self` je konkrétna hodnota.
- `new` je bežný názov pridruženej funkcie, nie špeciálne kľúčové slovo ani automatický konštruktor. Nemá parameter `self`, preto sa volá cez `ReplayBuffer::new(...)`.
- `max_blocks,` vo výraze štruktúry je skratka za `max_blocks: max_blocks,`.
- `push(&mut self, ...)` si výhradne požičia buffer na zmenu a prevezme nový blok hodnotou. `Arc::new` vytvorí jeho zdieľané vlastníctvo.
- `snapshot(&self)` si buffer požičia na čítanie. Vrátený snapshot vlastní nové `Arc` odkazy, takže nie je požičaním viazaný na ďalší život buffera.
- Kontrola nulovej kapacity pri vytvorení zabráni neplatnému stavu cez túto funkciu. Polia sú bez `pub`, ale kód v rovnakom module k nim stále môže pristupovať; hranicu súkromia sprísnime oddelením modulu neskôr.

V `main` nahradíme priamu manipuláciu s `VecDeque`:

```rust
let mut buffer = ReplayBuffer::new(3).expect("Používame kladnú kapacitu");

for id in 1..=3 {
    buffer.push(VideoBlock {
        id,
        data: vec![0u8; 1024],
    });
}

let snapshot = buffer.snapshot();
```

V priebežnej slučke odstránime ručné `len()` / `pop_front()` a nahradíme `push_back(Arc::new(...))`:

```rust
buffer.push(VideoBlock {
    id: next_id,
    data: vec![0u8; 1024],
});
```

Pracovník, `process_snapshot` a kanál zostávajú rovnaké. `expect` tu rozbaľuje vytvorenie so známou kladnou konštantou; pri konfigurácii dodanej používateľom chybu spracujeme. Ručne overíme cez `cargo run --example buffer`: úspešný výsledok zostáva 3 072 bajtov. Kapacita kolekcie nie je limit spotreby RAM celej aplikácie: snapshoty môžu stále držať už vyradené bloky a veľkosti dát blokov sa môžu líšiť.

## Jednotkové testy buffera

Testy overujú správanie bez ručného čítania výpisov. Tento blok pridaj na koniec `examples/buffer.rs`, mimo `main`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn block(id: u32) -> VideoBlock {
        VideoBlock {
            id,
            data: vec![42u8; 4],
        }
    }

    #[test]
    fn keeps_newest_blocks() {
        let mut buffer = ReplayBuffer::new(2).unwrap();

        for id in 1..=3 {
            buffer.push(block(id));
        }

        let ids: Vec<u32> = buffer.snapshot()
            .iter()
            .map(|block| block.id)
            .collect();

        assert_eq!(ids, vec![2, 3]);
    }

    #[test]
    fn snapshot_shares_data_and_survives_eviction() {
        let mut buffer = ReplayBuffer::new(1).unwrap();
        buffer.push(block(1));

        let snapshot = buffer.snapshot();
        let another_snapshot = buffer.snapshot();
        assert!(Arc::ptr_eq(&snapshot[0], &another_snapshot[0]));
        drop(another_snapshot);

        buffer.push(block(2));

        assert_eq!(buffer.snapshot()[0].id, 2);
        assert_eq!(snapshot[0].id, 1);
        assert_eq!(snapshot[0].data, vec![42u8; 4]);
    }

    #[test]
    fn rejects_zero_capacity() {
        assert!(ReplayBuffer::new(0).is_err());
    }
}
```

Spustenie testov konkrétneho example:

```powershell
cargo test --example buffer
```

- `#[cfg(test)]` zahrnie modul iba pri kompilácii pre testy.
- `mod tests { ... }` je vnorený modul. `use super::*` sprístupní názvy z rodičovského modulu, napríklad `ReplayBuffer`, `VideoBlock` a `Arc`.
- `#[test]` označí funkciu, ktorú spustí testovací runner. Pri tomto príkaze sa nespúšťa pôvodné `main` príkladu.
- `assert!(condition)` zlyhá, ak podmienka neplatí. `assert_eq!(actual, expected)` porovná hodnoty a pri rozdiele vypíše obe.
- `Arc::ptr_eq` overuje rovnakú alokáciu, nie iba rovnaký obsah. Tak kontrolujeme, že snapshoty skutočne zdieľajú blok.
- `unwrap()` v teste vyberie úspešnú hodnotu; pri neočakávanej chybe test zlyhá. Neznamená to, že tak máme spracúvať bežné prevádzkové chyby aplikácie.
- Pomocná funkcia `block` iba vytvára malé skúšobné dáta. Žiadny z týchto testov nezapisuje video ani nespúšťa pracovné vlákno.

Očakávaný výsledok po pridaní je `3 passed; 0 failed`. Testy overujú vyraďovanie najstaršieho bloku, zachovanie a zdieľanie dát snapshotu a odmietnutie nulovej kapacity; neoverujú reálne snímanie ani kódovanie videa.

## Časový buffer: Duration a zachovanie celého úseku

Nasledujúci krok nahrádza limit počtu blokov cieľovou dĺžkou videa. `Duration` predstavuje časový interval, nie konkrétny dátum alebo okamih. Už ho importujeme pre `thread::sleep`.

Predpoklad učebného modelu: bloky obsahujú kladné dĺžky videa a nadväzujú na seba bez medzier alebo prekryvov. Súčet ich dĺžok preto predstavuje pokrytý úsek. Čas videa sa neodvodzuje zo spánku vlákna. V reálnom enkodéri budeme pracovať s časovými údajmi média a hranicami kľúčových snímok.

Celé bloky sa nedajú ľubovoľne rozdeliť. Pre cieľ 5 sekúnd a dvojsekundové bloky ponecháme 6 sekúnd: odstránením najstaršieho by zostali iba 4. Buffer tak uchová celý požadovaný úsek aj s potrebným presahom. Presná dĺžka exportu je samostatná úloha; tento krok ešte negarantuje presne päťsekundový výsledný súbor. Krátko po štarte uchovávame všetko dostupné.

Nahraď definície `VideoBlock`, `ReplayBuffer` a jeho `impl`:

```rust
#[derive(Debug)]
struct VideoBlock {
    id: u32,
    data: Vec<u8>,
    duration: Duration,
}

struct ReplayBuffer {
    blocks: VecDeque<Arc<VideoBlock>>,
    target_duration: Duration,
    total_duration: Duration,
}

impl ReplayBuffer {
    fn new(target_duration: Duration) -> Result<Self, String> {
        if target_duration.is_zero() {
            return Err("Dĺžka buffera musí byť väčšia ako nula".to_string());
        }

        Ok(Self {
            blocks: VecDeque::new(),
            target_duration,
            total_duration: Duration::ZERO,
        })
    }

    fn push(&mut self, block: VideoBlock) -> Result<(), String> {
        if block.duration.is_zero() {
            return Err("Blok musí mať kladnú dĺžku".to_string());
        }

        self.total_duration = self.total_duration
            .checked_add(block.duration)
            .ok_or("Súčet dĺžok je príliš veľký".to_string())?;

        self.blocks.push_back(Arc::new(block));

        while let Some(oldest) = self.blocks.front() {
            let remaining = self.total_duration - oldest.duration;

            if remaining < self.target_duration {
                break;
            }

            self.blocks.pop_front();
            self.total_duration = remaining;
        }

        Ok(())
    }

    fn snapshot(&self) -> Vec<Arc<VideoBlock>> {
        self.blocks.iter().map(Arc::clone).collect()
    }
}
```

- `Duration::from_secs(3)` vytvorí interval troch sekúnd; `from_millis(500)` pol sekundy; `Duration::ZERO` nulu.
- `checked_add` vráti `Option<Duration>`: pri pretečení rozsahu `None`. `ok_or(...)` prevedie `Option` na `Result`; `?` vráti chybu volajúcemu. Odmietnutý blok tak nezmení obsah ani evidovaný čas.
- `while let Some(oldest) = ...` opakuje telo, kým predná položka existuje a neprerušíme cyklus.
- `front()` si najstarší blok iba požičia. Po výpočte `remaining` už referenciu nepoužívame, takže Rust dovolí meniť kolekciu cez `pop_front()` (NLL).
- Najstarší blok odstránime iba vtedy, ak aj bez neho zostáva aspoň požadovaná dĺžka. Pri dostatku dát teda platí, že odstránenie ďalšieho celého bloku by už skrátilo buffer pod cieľ.
- Počet blokov teraz nepoznáme vopred, preto začíname s `VecDeque::new()`. Kolekcia môže počas rastu zväčšovať kapacitu; video dáta vo vnútorných alokáciách tým nekopírujeme.
- Tento časový limit nie je pevný limit RAM. Závisí aj od bitrate, dĺžky blokov a aktívnych snapshotov.

Úpravy volajúcich v `main` aj testoch:

1. Každé `ReplayBuffer::new(n)` zmeň na `ReplayBuffer::new(Duration::from_secs(n))`. Test nuly môže použiť `Duration::ZERO`.
2. Do každého vytvoreného `VideoBlock` doplň `duration: Duration::from_secs(1)`, vrátane pomocnej funkcie `block` v testoch.
3. `push` teraz vracia `Result`: v testoch používaj `buffer.push(block(id)).unwrap()`, pri našich konštantných skúšobných dátach v `main` pripoj `.expect("Platný skúšobný blok")`.
4. Funkcia `process_snapshot`, kanál a snapshot zostávajú bez zmeny. Pôvodné tri testy majú ďalej platiť pre sekundové bloky; test nuly môžeš premenovať na `rejects_zero_duration`.

Príklad vloženia po úprave:

```rust
buffer.push(VideoBlock {
    id: next_id,
    data: vec![0u8; 1024],
    duration: Duration::from_secs(1),
}).expect("Platný skúšobný blok");
```

Do `mod tests` pridaj kontrolu hranice a odmietnutia neplatného bloku:

```rust
#[test]
fn keeps_whole_blocks_covering_target_duration() {
    let mut buffer = ReplayBuffer::new(Duration::from_secs(5)).unwrap();

    for id in 1..=4 {
        let mut item = block(id);
        item.duration = Duration::from_secs(2);
        buffer.push(item).unwrap();
    }

    let snapshot = buffer.snapshot();
    let ids: Vec<u32> = snapshot.iter().map(|item| item.id).collect();
    let duration: Duration = snapshot.iter().map(|item| item.duration).sum();

    assert_eq!(ids, vec![2, 3, 4]);
    assert_eq!(duration, Duration::from_secs(6));
}

#[test]
fn rejects_zero_length_block_without_changing_buffer() {
    let mut buffer = ReplayBuffer::new(Duration::from_secs(1)).unwrap();
    buffer.push(block(1)).unwrap();

    let mut invalid = block(2);
    invalid.duration = Duration::ZERO;
    assert!(buffer.push(invalid).is_err());

    let snapshot = buffer.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].id, 1);

    buffer.push(block(3)).unwrap();
    assert_eq!(buffer.snapshot()[0].id, 3);
}
```

`.sum()` spotrebuje iterátor a spočíta hodnoty; výsledný typ tu určuje anotácia `Duration`. Over cez `cargo test --example buffer` (očakávaných päť testov po doplnení) a `cargo run --example buffer` (stále spracuje pôvodné tri bloky po 1 024 bajtov). Tieto očakávania sú návod pre ručné overenie, nie tvrdenie o už vykonanom teste upraveného kódu.

## Rôzne dĺžky blokov, dvojice a super

Pri cieli 5 sekúnd majme bloky A=1 s, B=2 s a C=2 s. Po pridaní D=4 s máme 9 s. Odstránime A (zostane 8 s) a B (zostane 6 s). C už ponecháme: bez neho by zostali iba 4 s. Toto overuje viacnásobné vyradenie pri jednom pridaní aj zachovanie celého požadovaného úseku.

Používateľ má pomocnú funkciu `block(id, duration)` v rodičovskom module a staršiu `block(id)` v `mod tests`. Lokálna funkcia v testoch zatieni importované meno. Cez `super::block` explicitne vyberieme rodičovskú funkciu; Rust funkcie nepreťažuje podľa počtu argumentov.

Do `mod tests` pridaj:

```rust
#[test]
fn handles_blocks_with_different_durations() {
    let mut buffer = ReplayBuffer::new(Duration::from_secs(5)).unwrap();

    for (id, seconds) in [(1, 1), (2, 2), (3, 2), (4, 4)] {
        let item = super::block(id, Duration::from_secs(seconds));
        buffer.push(item).unwrap();
    }

    let snapshot = buffer.snapshot();
    let ids: Vec<u32> = snapshot.iter().map(|item| item.id).collect();
    let duration: Duration = snapshot.iter().map(|item| item.duration).sum();

    assert_eq!(ids, vec![3, 4]);
    assert_eq!(duration, Duration::from_secs(6));
}
```

`(1, 2)` je tuple (dvojica), `[(1, 1), (2, 2), ...]` je pole dvojíc a `for (id, seconds)` každú dvojicu rozbalí. Podobá sa to destructuringu v JS alebo structured bindings v C++. Typy sa tu odvodia z volaní na `u32` pre ID a `u64` pre počet sekúnd.

`super::` odkazuje na rodičovský modul, nie na nadradenú triedu. `use super::*` sprístupní rodičovské názvy, ale lokálna položka rovnakého mena má pred takýmto importom prednosť.

Over cez `cargo test --example buffer`. Pri aktuálnych štyroch testoch pred týmto krokom bude po pridaní tohto testu očakávaný počet päť.

## Moduly, knižnica a viditeľnosť pub

Keď buffer vyberieme z príkladu, môže rovnaký kód používať hlavná aplikácia aj učebný example. V balíku môžu súčasne existovať knižnica aj spustiteľné programy:

```text
src/
  lib.rs           # Koreň knižnice
  replay_buffer.rs # Typy VideoBlock a ReplayBuffer a implementácia buffera
  main.rs          # Hlavný spustiteľný program
examples/
  buffer.rs        # Samostatný učebný program používajúci knižnicu
```

Postup presunu:

1. Vytvor `src/replay_buffer.rs` a presuň sem z príkladu celé definície `VideoBlock`, `ReplayBuffer` a `impl ReplayBuffer`. Na začiatok pridaj `use std::{collections::VecDeque, sync::Arc, time::Duration};`.
2. Typy označ `pub struct VideoBlock` a `pub struct ReplayBuffer`. Polia `id`, `data`, `duration` vo `VideoBlock` označ `pub`, aby ich volajúci mohol vytvárať a čítať. Polia `ReplayBuffer` nechaj súkromné.
3. Metódy označ `pub fn new`, `pub fn push` a `pub fn snapshot`; ich telá nemeníme. Blok `impl ReplayBuffer` nemá vlastné `pub`.
4. Vytvor `src/lib.rs` s riadkom `pub mod replay_buffer;`.
5. V `examples/buffer.rs` odstráň presunuté definície a pridaj `use replay_recorder::replay_buffer::{ReplayBuffer, VideoBlock};`. Z importov štandardnej knižnice odstráň `collections::VecDeque`, ostatné zostávajú.
6. `process_snapshot`, pomocné funkcie, `main` a `mod tests` zatiaľ zostávajú v príklade. Hlavný `src/main.rs` zatiaľ nemusíš meniť.

```rust
// src/lib.rs
pub mod replay_buffer;
```

```rust
// Začiatok examples/buffer.rs po presune
use replay_recorder::replay_buffer::{ReplayBuffer, VideoBlock};
use std::{sync::{Arc, mpsc}, thread, time::Duration};
```

`mod` deklaruje modul a zapojí jeho zdroj do kompilácie. `use` sprístupní existujúci názov v aktuálnom rozsahu; nie je to textové vloženie súboru ako C++ `#include`. `pub mod` umožní pristupovať k modulu zvonka, ale jeho jednotlivé položky majú vlastnú viditeľnosť.

`pub struct` automaticky nesprístupní polia ani metódy. Preto explicitne sprístupníme API, kým `blocks`, `target_duration` a `total_duration` zostanú chránené hranicou modulu. Kód príkladu už nemôže obísť `push` a ľubovoľne meniť vnútornú kolekciu.

Názov balíka v Cargo je `replay-recorder`, ale predvolený názov knižničného crate v Rust kóde je `replay_recorder`. Cargo automaticky rozpozná `src/lib.rs`; tento krok nevyžaduje pridanie závislosti ani zmenu `Cargo.toml`. Example je samostatný crate používajúci túto knižnicu, preto potrebuje verejné položky.

Over po presune:

```powershell
cargo test --example buffer
cargo run --example buffer
```

Počet a výsledky testov majú zostať rovnaké. Testy teraz používajú knižničný buffer cez verejné API. Produkčné snímanie ani export týmto presunom ešte nevznikajú.

## Prepojenie hlavnej aplikácie s konfiguráciou

Buffer z knižnice začneme vytvárať podľa `config.toml`. Hlavný program zatiaľ len načíta a overí nastavenia, vytvorí prázdny buffer a skončí. Nenahráva obrazovku a nevytvára výstupné video.

V `src/config.rs` pridaj `use std::time::Duration;` a do existujúceho `impl Config` tieto metódy (validáciu nechaj):

```rust
pub fn recording_enabled(&self) -> bool {
    self.recording.enabled
}

pub fn replay_duration(&self) -> Duration {
    Duration::from_secs(self.recording.duration_seconds)
}
```

Sú to metódy poskytujúce údaje na čítanie bez sprístupnenia polí na priamu zmenu. `recording_enabled` vracia kópiu `bool`; `replay_duration` zostaví časový interval z číselnej hodnoty v konfigurácii. Metódy si konfiguráciu požičiavajú cez `&self`.

Obsah `src/main.rs` nahraď:

```rust
mod config;

use std::{error::Error, process::ExitCode};
use replay_recorder::replay_buffer::ReplayBuffer;
use config::load_config;

fn run() -> Result<(), Box<dyn Error>> {
    let config = load_config()?;

    if !config.recording_enabled() {
        println!("Nahrávanie je v konfigurácii vypnuté.");
        return Ok(());
    }

    let duration = config.replay_duration();
    let buffer = ReplayBuffer::new(duration)?;

    println!(
        "Inicializovaný prázdny buffer na {} sekúnd ({} blokov).",
        duration.as_secs(),
        buffer.snapshot().len()
    );

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Spustenie aplikácie zlyhalo: {error}");
            ExitCode::FAILURE
        }
    }
}
```

- `run` je obyčajná funkcia; názov nie je špeciálny. Vďaka jej návratovému typu vieme použiť `?` pre inicializačné kroky a spracovať chybu na jednom mieste v `main`.
- `!` pred hodnotou typu `bool` znamená logickú negáciu. Je to iné použitie než `!` za názvom makra v `println!`.
- `duration.as_secs()` vráti počet celých sekúnd. `Duration` je `Copy`, takže po odovzdaní do `new` ho môžeme ďalej použiť.
- V `main.rs` už nedeklaruj `mod replay_buffer;`: modul je deklarovaný v `lib.rs` a používame ho cez knižnicu `replay_recorder`. Druhá deklarácia by kompilovala rovnaký súbor znovu ako samostatný modul binárneho crate s vlastnými typmi.
- `mod config;` zatiaľ zostáva: konfigurácia patrí aktuálne binárnemu crate. `crate::config::Config` v loaderi preto stále funguje.
- Na konci `run` sa buffer zruší, pretože zatiaľ nemáme hlavnú slučku aplikácie. Hlásenie oznamuje inicializáciu, nie aktívne nahrávanie.

Ručne over z koreňového priečinka projektu: `cargo run` pri `duration_seconds = 300` vypíše 300 sekúnd a 0 blokov; pri `enabled = false` oznámi vypnutie; pri `duration_seconds = 0` vráti chybu validácie a neúspešný exit code. Loader validuje celú konfiguráciu aj pri vypnutom nahrávaní. Po skúške vráť platnú konfiguráciu. `cargo test --example buffer` ďalej overuje knižničný buffer.

## Príkazy počas behu: konzola a enum

Samostatný učebný príklad `examples/commands.rs` načítava príkazy z konzoly v jednom vlákne a posiela ich hlavnému vláknu cez kanál. `s` + Enter znamená požiadavku na uloženie, `q` + Enter ukončenie. Zatiaľ sa nevytvára snapshot, nezapisuje súbor a neregistruje globálna klávesová skratka.

```rust
use std::{
    io::{self, BufRead},
    sync::mpsc,
    thread,
};

enum AppCommand {
    SaveReplay,
    Quit,
}

fn main() {
    let (sender, receiver) = mpsc::channel::<AppCommand>();

    println!("Príkazy: s + Enter = požiadavka na uloženie, q + Enter = koniec");

    let input = thread::spawn(move || {
        let stdin = io::stdin();

        for line in stdin.lock().lines() {
            let line = match line {
                Ok(line) => line,
                Err(error) => {
                    eprintln!("Chyba čítania vstupu: {error}");
                    break;
                }
            };

            match line.trim() {
                "s" => {
                    if sender.send(AppCommand::SaveReplay).is_err() {
                        break;
                    }
                }
                "q" => {
                    let _ = sender.send(AppCommand::Quit);
                    break;
                }
                _ => println!("Neznámy príkaz. Použi s alebo q."),
            }
        }
    });

    while let Ok(command) = receiver.recv() {
        match command {
            AppCommand::SaveReplay => {
                println!("Prijatá požiadavka na uloženie replaya.");
            }
            AppCommand::Quit => break,
        }
    }

    input.join().expect("Vlákno vstupu skončilo panikou");
    println!("Aplikácia skončila.");
}
```

Spusti cez `cargo run --example commands`, zadaj napríklad `s`, znovu `s` a potom `q`, vždy s Enterom.

- `enum AppCommand` definuje povolené správy. Text z konzoly prevedieme na variant enumu ešte pred odoslaním; ďalší kód už neporovnáva textové príkazy.
- `io::stdin()` poskytne štandardný vstup. `lock()` ho uzamkne pre čítanie; `lines()` je metóda traitu `BufRead`, ktorý preto importujeme.
- Každý prvok z `lines()` je `Result<String, io::Error>`. Čítanie môže čakať na používateľa, ale čaká vo vstupnom vlákne.
- `trim()` vráti požičaný text bez okrajových whitespace znakov; nevytvára nový `String`.
- `_` vo vetve `match` zachytí všetky ostatné hodnoty.
- `let _ = sender.send(...)` vedome ignoruje výsledok. Pri odchode je to prijateľné: ak príjemca už neexistuje, vstupné vlákno aj tak skončí. Nejde o odporúčaný spôsob ignorovania chýb zápisu videa.
- `recv()` blokuje hlavné vlákno, kým príde príkaz alebo sa odpojí kanál. V tomto samostatnom príklade nemá hlavné vlákno inú prácu. Pri spojení s recorderom musíme obslúžiť aj nové bloky a výsledky ukladania; nekopírujeme túto slučku bez úpravy do tray UI.
- `while let Ok(...)` skončí pri odpojení kanála. Pri konci vstupu (EOF) vstupné vlákno odíde z cyklu a zruší posledný sender, takže sa ukončí aj príjem.
- Po `q` vstupné vlákno prestane čítať a skončí, preto tu môžeme zavolať `join()`. Pri budúcom ukončení vyvolanom iným zdrojom by vlákno čakajúce na konzolový vstup nemuselo skončiť samo; tray aplikácia bude mať iný životný cyklus vstupu.

Tento príklad pripravuje oddelenie zdroja príkazov od spracovania. Globálna skratka a tray menu neskôr môžu odosielať rovnaké `AppCommand::SaveReplay`, ale ich konkrétne napojenie ešte nie je implementované.

## Jedna pracovná úloha naraz: Option a JoinHandle

Tento krok rozširuje `examples/commands.rs`. Naplníme buffer tromi skúšobnými blokmi a na `s` vytvoríme snapshot pre pracovné vlákno. Počas jeho práce ďalšie `s` odmietneme. Stále ide o simuláciu: buffer v tomto príklade neprijíma priebežné video a pracovník nezapisuje súbor.

Pridaj importy:

```rust
use replay_recorder::replay_buffer::{ReplayBuffer, VideoBlock};
use std::time::Duration;
```

Na začiatku `main` pred spustením vstupného vlákna priprav:

```rust
let mut buffer = ReplayBuffer::new(Duration::from_secs(3))
    .expect("Platná dĺžka buffera");

for id in 1..=3 {
    buffer.push(VideoBlock {
        id,
        data: vec![42u8; 4],
        duration: Duration::from_secs(1),
    }).expect("Platný skúšobný blok");
}

let mut worker: Option<thread::JoinHandle<()>> = None;
```

V hlavnom `match command` nahraď vetvu `AppCommand::SaveReplay`:

```rust
AppCommand::SaveReplay => {
    let busy = worker.as_ref()
        .is_some_and(|handle| !handle.is_finished());

    if busy {
        println!("Úloha už beží, ďalšiu nespúšťam.");
        continue;
    }

    if let Some(handle) = worker.take() {
        if handle.join().is_err() {
            eprintln!("Predchádzajúce pracovné vlákno skončilo panikou.");
        }
    }

    let snapshot = buffer.snapshot();

    worker = Some(thread::spawn(move || {
        let bytes: usize = snapshot.iter()
            .map(|block| block.data.len())
            .sum();

        thread::sleep(Duration::from_secs(2));
        println!("Simulácia dokončená: {bytes} bajtov.");
    }));
}
```

Za hlavný prijímací cyklus, pred záverečné hlásenie o ukončení, pridaj čakanie na prípadnú zostávajúcu úlohu:

```rust
if let Some(handle) = worker {
    println!("Dokončujem pracovné vlákno pred ukončením.");
    if handle.join().is_err() {
        eprintln!("Pracovné vlákno skončilo panikou.");
    }
}
```

Pôvodné `input.join()` ponechaj. Pri `q` vstupné vlákno už končí, takže toto čakanie je pre daný príklad vhodné.

- `Option<JoinHandle<()>>` vyjadruje, že handle buď chýba (`None`), alebo ho vlastníme (`Some`). Vnútorné `()` znamená, že pracovník nevracia užitočnú hodnotu.
- `Some(handle)` samo osebe neznamená, že vlákno ešte beží. Môže byť už skončené; stav zisťujeme cez `is_finished()`.
- `as_ref()` prevedie `&Option<T>` na `Option<&T>` a dovolí skontrolovať handle bez prevzatia vlastníctva.
- `is_some_and(...)` vráti `true`, iba ak existuje hodnota a closure pre ňu vráti `true`.
- `worker.take()` vyberie vlastnený handle a na jeho miesto vloží `None`. Potom ho môžeme spotrebovať cez `join()`.
- `continue` preskočí zvyšok aktuálnej iterácie prijímacieho cyklu a čaká na ďalší príkaz.
- Dokončený handle neudržiava snapshot pri živote. Snapshot zaniká pri skončení closure, hoci handle ešte zostáva uložený v `worker`.
- Pri bežnom príkaze voláme `join()` iba na už skončenom vlákne. Pri odchode naň zámerne čakáme, aby program neprerušil rozpracovanú úlohu.
- Výpis robí zatiaľ samotný pracovník. V aplikácii sa výsledok vráti do riadiaceho vlákna, ktoré aktualizuje používateľský stav; spracovanie `Result` cez kanál sme si ukázali samostatne.

Ručné overenie cez `cargo run --example commands`: zadaj `s`, hneď ďalšie `s` (má sa odmietnuť), po skončení simulácie znovu `s` (má sa prijať) a počas práce `q` (program má počkať na dokončenie). Pri troch štvorbajtových blokoch simulácia oznámi 12 bajtov. Uložením sa buffer nevyprázdňuje.

## Priebežné dopĺňanie: Instant a recv_timeout

V `examples/commands.rs` necháme pôvodné tri skúšobné bloky a počas behu pridávame ďalší približne každú sekundu. Je to simulátor, nie snímanie obrazovky. Deklarovaná sekundová dĺžka bloku je skúšobná hodnota; pri oneskorení slučky nedorábame zmeškané snímky ani negarantujeme presný čas videa.

Import `time::Duration` rozšír na `time::{Duration, Instant}`. Tesne pred prijímací cyklus vlož:

```rust
let block_interval = Duration::from_secs(1);
let mut next_block_at = Instant::now() + block_interval;
let mut next_id = 4;
```

Nasledujúci úryvok je zámerne iba začiatok cyklu: nahrádza riadok `while let Ok(command) = receiver.recv() {`. Existujúci `match command` a jeho telo nasledujú hneď za úryvkom, pôvodná zatváracia zátvorka cyklu zostane na mieste.

```rust
loop {
    let now = Instant::now();

    if now >= next_block_at {
        buffer.push(VideoBlock {
            id: next_id,
            data: vec![42u8; 4],
            duration: block_interval,
        }).expect("Platný skúšobný blok");

        println!("Buffer prijal blok {next_id}");
        next_id += 1;
        next_block_at = now + block_interval;
    }

    let wait = next_block_at.saturating_duration_since(Instant::now());

    let command = match receiver.recv_timeout(wait) {
        Ok(command) => command,
        Err(mpsc::RecvTimeoutError::Timeout) => continue,
        Err(mpsc::RecvTimeoutError::Disconnected) => break,
    };
```

Pre viditeľnú kontrolu snapshotu pridaj do pracovnej closure pred dvojsekundový spánok:

```rust
let ids: Vec<u32> = snapshot.iter().map(|block| block.id).collect();
println!("Pracovník začal so snapshotom {ids:?}");
```

Za spánok môžeš pridať:

```rust
println!("Pracovník dokončil snapshot {ids:?}");
```

- `Duration` je dĺžka intervalu, `Instant` je monotónny časový bod vhodný na meranie a plánovanie. Nie je to kalendárny dátum.
- `recv_timeout(wait)` čaká na správu najviac určený interval. Pri správe vráti `Ok`, pri uplynutí intervalu `Timeout`, pri prázdnom odpojenom kanáli `Disconnected`.
- `saturating_duration_since` vráti nulu, ak už termín uplynul. Tak môžeme bezpečne vyžiadať okamžitú kontrolu kanála.
- Termín ďalšieho bloku držíme oddelene od prijímania príkazov. Každé prijaté `s` preto nezačne novú celú sekundu čakania a opakované príkazy neodkladajú dopĺňanie donekonečna.
- Hlavné vlákno spravuje buffer a príkazy; pracovné vlákno pracuje iba so snapshotom. Počas dvojsekundovej práce hlavná slučka pokračuje.
- Po `q` už nové bloky nepribúdajú, ale záverečné `join()` počká na rozpracovanú úlohu.

Ručné overenie: `cargo run --example commands`, chvíľu čakaj bez vstupu (majú pribúdať bloky), potom `s` (snapshot má zachytiť aktuálne tri bloky), počas práce sleduj nové bloky a skús druhé `s` (odmietnutie). Po dokončení ďalšie `s` zachytí novšie ID. Konkrétne ID závisia od načasovania. Zápis videa ani aktívne snímanie týmto krokom ešte nevzniká.

## Prvý zápis na disk: skúšobný binárny súbor

V `examples/commands.rs` nahradíme dvojsekundovú simuláciu zápisom aktuálnych skúšobných bajtov do `.bin`. Tento súbor nie je video. Príklad zatiaľ používa z konfigurácie iba výstupný priečinok; formát MP4/MKV, FPS a bitrate sa na skúšobné bajty nevzťahujú. Zápis aj vytvorenie priečinka nastávajú až pri príkaze `s`.

V `src/config.rs` zmeň import na `use std::path::{Path, PathBuf};` a do `impl Config` pridaj:

```rust
pub fn output_directory(&self) -> &Path {
    &self.recording.output_directory
}
```

`&Path` je požičaný pohľad na cestu, podobne ako `&str` na `String`. `PathBuf` vlastní cestu. `.to_path_buf()` vytvorí jej vlastnenú kópiu.

Importy príkladu rozšír na:

```rust
use std::{
    fs::{self, OpenOptions},
    io::{self, BufRead, Write},
    path::PathBuf,
    sync::{Arc, mpsc},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use replay_recorder::{
    config::load_config,
    replay_buffer::{ReplayBuffer, VideoBlock},
};
```

Nad `main` pridaj:

```rust
fn save_snapshot(
    snapshot: Vec<Arc<VideoBlock>>,
    directory: PathBuf,
) -> io::Result<PathBuf> {
    if snapshot.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Snapshot je prázdny",
        ));
    }

    fs::create_dir_all(&directory)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();

    let path = directory.join(format!("replay-{timestamp}.bin"));

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;

    for block in snapshot {
        file.write_all(&block.data)?;
    }

    file.sync_all()?;
    Ok(path)
}
```

Na začiatok `main` pridaj:

```rust
let config = load_config().expect("Nepodarilo sa načítať konfiguráciu");
let output_directory = config.output_directory().to_path_buf();
```

V obsluhe `SaveReplay` nechaj kontrolu obsadenosti a dokončeného handle. Úsek od vytvorenia snapshotu po vytvorenie pracovníka nahraď:

```rust
let snapshot = buffer.snapshot();
let directory = output_directory.clone();

worker = Some(thread::spawn(move || {
    match save_snapshot(snapshot, directory) {
        Ok(path) => println!("Skúšobný súbor uložený: {}", path.display()),
        Err(error) => eprintln!("Zápis zlyhal: {error}"),
    }
}));
```

- `io::Result<PathBuf>` je alias pre `Result<PathBuf, io::Error>`.
- `fs::create_dir_all` vytvorí chýbajúce priečinky. Samotné načítanie cesty na začiatku programu ich nevytvára.
- `SystemTime` je systémový čas; tu slúži iba pre názov súboru. Na plánovanie intervalov stále používame `Instant`. `map_err` prevedie chybu času na I/O chybu našej funkcie.
- `.join(...)` na ceste zostaví cestu k súboru. Nesúvisí s `JoinHandle::join()` pri vláknach.
- `create_new(true)` zlyhá, ak súbor už existuje. Časová značka znižuje pravdepodobnosť kolízie názvu, ale samotnú ochranu proti prepísaniu zaručuje `create_new`.
- `write_all` z traitu `Write` sa pokúsi zapísať celý blok bajtov, prípadne vráti chybu. Jednoduché `write` môže zapísať iba časť.
- `file.sync_all()` požiada o synchronizáciu obsahu a metadát súboru s úložiskom a umožní zachytiť ďalšie chyby. Samotné uvoľnenie `File` nevie vrátiť chybu jeho uzatvorenia volajúcemu.
- `for block in snapshot` spotrebúva snapshot. Jednotlivé `Arc` odkazy sa uvoľňujú po zapísaní blokov; ak ich ešte drží buffer, dáta zostanú dostupné.
- `directory.clone()` kopíruje malú cestu, nie video. Hlavná slučka si ponechá pôvodnú cestu pre ďalšie požiadavky.
- `path.display()` poskytne formátovateľnú reprezentáciu cesty pre výpis.
- Už nepridávame umelý spánok. Skúšobný zápis 12 bajtov môže byť veľmi rýchly; ďalšie `s` po dokončení sa správne prijme.
- Pri chybe po vytvorení súboru môže na disku zostať neúplný `.bin`. Ukážka to oznámi ako chybu; bezpečné dokončenie a čistenie čiastočných výstupov je ďalší krok.

Ručné overenie: spusti `cargo run --example commands` z koreňa projektu, nechaj pribúdať bloky a skontroluj, že výstupný priečinok nedostáva nové súbory. Až po `s` + Enter má vzniknúť `replay-<čas>.bin` v `recording.output_directory`. Pri súčasných troch blokoch po štyroch bajtoch má mať 12 bajtov, všetky s hodnotou 42. Ďalšie uloženie vytvorí nový súbor. Existujúci `.bin` neprepisujeme a nepomenúvame `.mp4`: formát súboru určuje jeho obsah, nie prípona.

## Dokončený výstup a dočasný súbor cez tempfile

Výsledný `.bin` nechceme vytvoriť ešte pred dokončením zápisu. Po explicitnom `s` zapisujeme do jedného dočasného `.part` súboru v cieľovom priečinku a až po úspešnom zápise ho zachováme pod výsledným názvom. Priebežný video buffer zostáva v RAM; dočasný súbor nevzniká pri dopĺňaní buffera.

Pridaj závislosť z koreňa projektu:

```powershell
cargo add tempfile
```

Cargo aktualizuje `Cargo.toml` a vyrieši závislosti v `Cargo.lock`. Crate je jednotka kompilácie Rustu; balík `tempfile` tu poskytuje knižničný crate. Už používané `serde` a `toml` sú tiež externé závislosti.

V `examples/commands.rs` odstráň import `OpenOptions` a nahraď `save_snapshot`:

```rust
fn save_snapshot(
    snapshot: Vec<Arc<VideoBlock>>,
    directory: PathBuf,
) -> io::Result<PathBuf> {
    if snapshot.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Snapshot je prázdny",
        ));
    }

    fs::create_dir_all(&directory)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();

    let path = directory.join(format!("replay-{timestamp}.bin"));

    let mut temporary = tempfile::Builder::new()
        .prefix("replay-")
        .suffix(".part")
        .tempfile_in(&directory)?;

    for block in snapshot {
        temporary.write_all(&block.data)?;
    }

    temporary.as_file().sync_all()?;

    temporary.persist_noclobber(&path)
        .map_err(|error| error.error)?;

    Ok(path)
}
```

- `Builder` nastaví predponu a príponu; knižnica vytvorí medzi nimi náhodnú časť názvu. Vznikne `NamedTempFile`, ktorý vlastní súbor aj zodpovednosť za odstránenie dočasnej cesty.
- Súbor vzniká v rovnakom priečinku ako finálny výsledok; nepotrebujeme prenášať dáta medzi diskami.
- `NamedTempFile` implementuje `Write`, preto stále používame `write_all`. `as_file()` požičia vnútorný `File` pre `sync_all()`.
- Pri obyčajnom návrate cez `?` sa vlastníci zrušia a `NamedTempFile` sa pokúsi dočasný súbor odstrániť. Chyba mazania v deštruktore sa nevracia volajúcemu; pri násilnom ukončení procesu sa deštruktor vôbec nemusí vykonať.
- `persist_noclobber` prevezme vlastníctvo dočasného súboru a pri úspechu ho zachová pod cieľovou cestou. Existujúci cieľ nikdy neprepíše. Nejde o kopírovanie všetkých bajtov do druhého výstupného súboru.
- Pri chybe vracia `PersistError`, ktorý obsahuje I/O chybu aj dočasný súbor. `map_err(|error| error.error)` vyberie I/O chybu pre návratový typ; zostávajúci dočasný súbor sa pri rušení chyby pokúsi upratať.
- API negarantuje atómové odstránenie pôvodnej dočasnej cesty na každej platforme; môže zostať ďalší odkaz na rovnaké dáta. Tento krok tiež nie je zárukou prežitia všetkých zmien názvov pri výpadku napájania. Výsledný názov však udeľujeme až po zapísaní celého obsahu a po úspešnom `sync_all`.
- Predošlé `OpenOptions::create(true)` by mohlo otvoriť existujúci súbor a zapisovať cez jeho začiatok. `create_new(true)` existujúci súbor odmieta; pri novej implementácii ochranu cieľa zabezpečuje `persist_noclobber`.

Oficiálna dokumentácia overeného API: [NamedTempFile a persist_noclobber](https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html#method.persist_noclobber).

Ručné overenie po úprave: `cargo check --example commands`, `cargo test --example buffer`, potom `cargo run --example commands`. Po `s` má vzniknúť hotový dvanásťbajtový `.bin`; pri bežnom úspešnom dokončení na Windows sa `.part` presunie na výslednú cestu. Pri takom malom obsahu môže byť `.part` viditeľný len veľmi krátko. Stále ukladáme skúšobné bajty, nie prehrateľné video.

## Prvá skutočná snímka do RAM: FFmpeg a Command

**Zmena smerovania:** tento príklad ponechávame ako voliteľnú ukážku práce s externým procesom. Používateľ chce natívnu Windows aplikáciu bez nutnosti inštalovať FFmpeg; ďalšie snímanie budeme riešiť cez Windows API, pozri nasledujúcu sekciu o natívnom snímaní. Ukážka FFmpeg nie je podmienkou pokračovania.

Samostatný príklad `examples/capture.rs` spustí externý FFmpeg, získa jedinú snímku Windows plochy ako PNG cez stdout a ponechá jej bajty vo `Vec<u8>`. Zatiaľ ich nezapája do replay buffera ani nezapisuje obrazový súbor.

Predpoklad: dostupný `ffmpeg.exe` so zariadením `gdigrab` a PNG enkodérom. Pri príprave tohto kroku nebol príkaz `ffmpeg` nájdený v PATH prostredia agenta; to nepreukazuje, že executable nie je inde na disku. Windows balíky sú odkazované na [oficiálnej stránke FFmpeg](https://ffmpeg.org/download.html#build-windows). Môžeme použiť rozbalený balík a explicitnú cestu bez zmeny PATH.

```rust
use std::{env, io, process::{Command, Stdio}};

fn main() -> io::Result<()> {
    let ffmpeg = env::args_os()
        .nth(1)
        .unwrap_or_else(|| "ffmpeg".into());

    let output = Command::new(ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel", "error",
            "-nostdin",
            "-f", "gdigrab",
            "-framerate", "1",
            "-i", "desktop",
            "-frames:v", "1",
            "-c:v", "png",
            "-f", "image2pipe",
            "pipe:1",
        ])
        .stdin(Stdio::null())
        .output()?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "FFmpeg zlyhal: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let png = output.stdout;

    if !png.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err(io::Error::other("Výstup nemá očakávanú PNG hlavičku"));
    }

    println!("Snímka plochy je v RAM: {} bajtov PNG.", png.len());
    Ok(())
}
```

Spusti s cestou prispôsobenou umiestneniu rozbaleného executable:

```powershell
cargo run --example capture -- "C:\Tools\ffmpeg\bin\ffmpeg.exe"
```

Ak je executable dostupný v PATH, stačí `cargo run --example capture`. Samostatné `--` oddeľuje argumenty Cargo od argumentov spúšťaného programu. FFmpeg je externý proces, nie nová Cargo závislosť.

- `env::args_os()` číta argumenty ako `OsString`, čo je vhodné aj pre systémové cesty. Argument 0 je názov programu, `nth(1)` vyberie prvý používateľský argument. `unwrap_or_else` poskytne predvolený názov, ak argument chýba.
- `Command::new` pripraví proces a `.args([...])` odovzdá jednotlivé argumenty bez skladania shellového príkazu.
- `.output()` proces spustí, počká na jeho koniec a zachytí stdout aj stderr do pamäte. `?` ošetrí zlyhanie spustenia alebo I/O; samostatná kontrola `status.success()` ošetrí neúspešný návratový kód FFmpegu.
- `.stdin(Stdio::null())` a `-nostdin` zabránia FFmpegu čítať príkazy z našej konzoly.
- `output.stdout` obsahuje binárne PNG dáta; neprevádzame ich na `String`. Textové diagnostiky sú v `output.stderr`; `from_utf8_lossy` ich umožní vypísať aj pri neplatných UTF-8 bajtoch.
- `b"..."` je bajtový literál. `starts_with` tu kontroluje iba PNG signatúru, nie úplnú platnosť a dekódovateľnosť obrázka.
- `gdigrab` + `desktop` sníma Windows plochu, `-frames:v 1` obmedzí výstup na jednu snímku, `png` ju zakóduje a `image2pipe` + `pipe:1` pošle výsledok na stdout namiesto obrazového súboru.
- Tento príklad je prvé overenie snímania a komunikácie s procesom. PNG snímky ešte nie sú navrhnuté komprimované video bloky a nesmieme ich iba zlepiť a nazvať MP4.
- `.output()` je tu vhodné vďaka konečnému výstupu jednej snímky. Na nepretržité nahrávanie použijeme priebežné čítanie procesu; inak by sa jeho celý výstup hromadil v RAM mimo limitu buffera.
- Po skončení príkladu `png` zanikne. Snímka sa nikam neukladá a počas prípravy návodu nebola živá captura overená, keďže FFmpeg nebol dostupný cez PATH.

Oficiálne zdroje: [gdigrab](https://ffmpeg.org/ffmpeg-devices.html#gdigrab), [pipe](https://ffmpeg.org/ffmpeg-protocols.html#pipe), [image2 a image2pipe](https://ffmpeg.org/ffmpeg-formats.html#image2_002c-image2pipe).

## Natívna Windows aplikácia bez externého FFmpeg

Požiadavka: používateľ výslednej aplikácie nemá inštalovať FFmpeg. Snímanie a kódovanie chceme napojiť na Windows API; Cargo závislosti sú záležitosťou zostavenia aplikácie, nie ručnej inštalácie Rust balíkov používateľom.

Navrhovaná cesta:

1. Windows Graphics Capture (prípadne Desktop Duplication API) získa obrazové snímky.
2. Enkodér cez Media Foundation ich komprimuje; prvý cieľ je H.264.
3. Replay buffer uchová komprimované dáta v RAM s časovými údajmi a informáciami potrebnými na dekódovanie.
4. Skratka vytvorí snapshot a export zostaví prehrateľný súbor; prvý cieľ je MP4.

Microsoft poskytuje Rust crate `windows` pre prístup k systémovým API. Pre prvé snímanie môžeme použiť vyššiu vrstvu `windows-capture`, aby sme sa nemuseli hneď učiť celú správu Direct3D a COM. Nie je to spúšťanie `ffmpeg.exe`.

Pri overovaní dokumentácie `windows-capture` 2.0.1 knižnica dokumentovala aj `VideoEncoder::new_from_stream` a použitie pamäťového streamu. **Pamäťový stream sám nie je rotačný buffer.** Treba ešte overiť, či a ako vieme získavať vhodné samostatné komprimované bloky, riadiť kľúčové snímky, zachovať časovanie a exportovať snapshot bez prerušenia kontinuálneho snímania. Zatiaľ netvrdíme, že existujúci `VecDeque<Arc<VideoBlock>>` možno bez úprav zapojiť priamo za tento enkodér.

Hardvérové kódovanie závisí od dostupného enkodéra a ovládačov; samotné natívne snímanie ho negarantuje. Cieľom distribúcie je EXE bez požiadavky na Rust, Cargo alebo samostatnú inštaláciu FFmpegu. Podporované verzie Windows, systémové mediálne súčasti a prípadné runtime závislosti overíme pri zostavení a distribučnom teste.

Najbližší učebný krok: získať prvú snímku cez natívne Windows snímanie a overiť rozmery/prístup k snímke. Výber a zapojenie video enkodéra nasleduje samostatne. Doterajší buffer, ownership, snapshoty, vlákna a riadenie zápisu zostávajú použiteľným základom, zatiaľ s učebnými dátami.

Overené primárne zdroje: [Rust for Windows](https://learn.microsoft.com/en-us/windows/dev-environment/rust/rust-for-windows), [Windows Graphics Capture](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture), [Media Foundation H.264 encoder](https://learn.microsoft.com/en-us/windows/win32/medfound/h-264-video-encoder), [windows-capture](https://docs.rs/crate/windows-capture/latest).

## Postupný prechod na NVIDIA NVENC

Dohodnutý smer: najprv vyskúšať `windows-capture` a jeho dostupné možnosti kódovania; po získaní skúseností pridať priamy enkodér cez NVIDIA Video Codec SDK, ak ho podporuje GPU a ovládač. Neimplementujeme NVENC ani zložitú všeobecnú architektúru vopred.

Snímanie a kompresia sú dve oddelené úlohy. `windows-capture` môže zostať zdrojom snímok aj po pridaní NVENC. NVIDIA Video Codec SDK prijíma obrazové dáta a produkuje komprimovaný bitstream; nenahrádza samo osebe snímanie plochy ani zostavenie MP4/MKV.

```text
windows-capture → enkodér → komprimované bloky v RAM → export
                    ├─ najprv: enkodér cez Windows
                    └─ neskôr: priamy NVENC, keď je dostupný
```

Praktické hranice kódu: snímanie, kódovanie, replay buffer a export. Spoločné rozhranie enkodéra cez Rust trait navrhneme až podľa reálneho toku snímok a paketov. Dnešný `VideoBlock { id, data, duration }` je učebný model; pre reálne video pribudne časovanie, označenie kľúčových snímok a konfigurácia kodeku potrebná pre prehrateľný export.

Pri neskoršom výbere NVENC budeme overovať dostupnosť API, konkrétny kodek, formát vstupu a podporované parametre, nie iba značku GPU. Ak nie sú splnené požiadavky, použijeme dostupnú pôvodnú cestu cez Windows a používateľovi pravdivo oznámime zvolený enkodér. Dostupnosť tejto záložnej cesty treba tiež overiť.

`windows-capture` už podporuje hardvérovú akceleráciu svojho enkodéra; priamy NVENC preto znamená najmä väčšiu kontrolu a možnosti optimalizácie, nie automaticky prvé využitie GPU ani garantované zrýchlenie. Pri integrácii preveríme odovzdávanie Direct3D textúr a kompatibilitu zariadení, aby sme zbytočne nekopírovali každú snímku cez CPU.

NVENC runtime poskytuje NVIDIA ovládač; používateľ našej hotovej aplikácie nemá inštalovať samotné vývojové SDK. Požadovaná verzia ovládača závisí od použitej verzie API a funkcií. Rust väzby na SDK aj distribúciu overíme pri konkrétnej implementácii.

Primárne zdroje: [NVENC programming guide](https://docs.nvidia.com/video-technologies/video-codec-sdk/13.1/nvenc-video-encoder-api-prog-guide/), [windows-capture encoder](https://docs.rs/crate/windows-capture/latest/source/src/encoder.rs).

## Prvá snímka cez windows-capture: implementácia traitu

Cieľ učebného príkladu: vybrať obrazovku alebo okno, prijať snímku, vypísať rozmery a zastaviť snímanie. Nepíšeme na disk ani nezapíname enkodér. API príklad vychádza z dokumentácie `windows-capture` 2.0.1.

```powershell
cargo add windows-capture@2.0.1
```

Vytvor `examples/native_capture.rs`:

```rust
use windows_capture::{
    capture::{Context, GraphicsCaptureApiHandler},
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    graphics_capture_picker::{Error as PickerError, GraphicsCapturePicker},
    settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings,
        DrawBorderSettings, MinimumUpdateIntervalSettings,
        SecondaryWindowSettings, Settings,
    },
};

type CaptureError = Box<dyn std::error::Error + Send + Sync>;

struct Capture;

impl GraphicsCaptureApiHandler for Capture {
    type Flags = ();
    type Error = CaptureError;

    fn new(_ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self)
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame<'_>,
        control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        println!("Prijatá snímka: {} × {}", frame.width(), frame.height());
        control.stop();
        Ok(())
    }
}

fn main() -> Result<(), CaptureError> {
    let item = match GraphicsCapturePicker::pick_item() {
        Ok(Some(item)) => item,
        Ok(None) | Err(PickerError::Canceled) => {
            println!("Výber bol zrušený.");
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };

    let settings = Settings::new(
        item,
        CursorCaptureSettings::Default,
        DrawBorderSettings::Default,
        SecondaryWindowSettings::Default,
        MinimumUpdateIntervalSettings::Default,
        DirtyRegionSettings::Default,
        ColorFormat::Rgba8,
        (),
    );

    Capture::start(settings)?;
    println!("Snímanie skončilo.");
    Ok(())
}
```

Spusti interaktívne v prihlásenej relácii Windows:

```powershell
cargo run --example native_capture
```

Vyber obrazovku alebo otvorené okno. Očakávame výpis rozmerov prvej prijatej snímky a ukončenie. Zrušenie výberu má skončiť bez chyby. V tejto lekcii overujeme príchod snímky a rozmery, nie prehrateľnosť videa ani GPU enkodér. Kompilácia a živé snímanie sú kroky na overenie po doplnení príkladu; počas písania lekcie nebol nový príklad zostavený ani spustený.

- `struct Capture;` je štruktúra bez polí (unit struct). Zatiaľ nepotrebuje vlastný stav; hodnota sa dá vytvoriť ako `Capture` alebo `Self` v jej implementácii.
- `impl GraphicsCaptureApiHandler for Capture` implementuje správanie požadované traitom pre náš typ. Je to podobné implementácii interface, nie dedenie implementácie ani pridanie polí rodičovskej triedy.
- `new` a `on_frame_arrived` vyžaduje trait. `start` už poskytuje knižnica ako metódu traitu, preto sme ju nemuseli napísať.
- `on_frame_arrived` je callback: pri príchode snímky ho zavolá knižnica. Náš kód ho sám v cykle nevolá.
- `type Flags = ();` a `type Error = CaptureError;` sú asociované typy traitu. Určujú typ kontextových údajov a chýb pre túto implementáciu. `()` znamená, že vlastné kontextové údaje zatiaľ neposielame; zodpovedá poslednému argumentu `Settings::new`.
- `type CaptureError = ...` mimo `impl` je alias typu, ktorý šetrí opakovanie dlhého zápisu. Sám nevytvorí odlišný nový typ chyby.
- `Send` vyjadruje možnosť bezpečne presunúť hodnotu medzi vláknami; `Sync` možnosť bezpečne zdieľať referencie na hodnotu medzi vláknami. Trait to vyžaduje pre typ chyby; tieto vlastnosti samy nevytvárajú vlákno ani zámok.
- `_ctx` je prijatý, zatiaľ nepoužívaný parameter; podčiarkovník potláča upozornenie na jeho nepoužitie.
- `Frame<'_>` má parameter životnosti, ktorý tu necháme odvodiť. `&mut Frame<'_>` je požičaný prístup k snímke, nie vlastnený `VideoBlock`. Referenciu nemôžeme svojvoľne odložiť do dlhodobého replay buffera.
- `Error as PickerError` premenuje importovaný názov, aby bolo jasné, ku ktorej časti patrí. `|` medzi vzormi v `match` znamená alternatívu: oba spôsoby ukončenia výberu spracujeme rovnako.
- `Capture::start(settings)` riadi aktuálne vlákno počas snímania. `control.stop()` po prijatí snímky požiada o zastavenie. Pri neskoršom tray UI zvolíme vhodné umiestnenie snímacej slučky samostatne.

Overené API: [GraphicsCaptureApiHandler](https://docs.rs/windows-capture/2.0.1/windows_capture/capture/trait.GraphicsCaptureApiHandler.html), [GraphicsCapturePicker](https://docs.rs/windows-capture/2.0.1/windows_capture/graphics_capture_picker/struct.GraphicsCapturePicker.html), [príklady nastavení](https://docs.rs/crate/windows-capture/2.0.1).

## Vlastnené pixely po skončení capture callbacku

Ďalší krok v `examples/native_capture.rs`: cez callback získame vlastnenú kópiu RGBA pixelov a kanálom ju odovzdáme do `main`. Rozmery a veľkosť vypíšeme až po návrate z `Capture::start`, takže dáta už nezávisia od požičanej snímky Windows. Stále ide o jedinú snímku, bez kódovania videa a bez zápisu na disk.

Pridaj import `use std::sync::mpsc;`. Namiesto `struct Capture;` vlož:

```rust
struct CapturedFrame {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

struct Capture {
    sender: mpsc::Sender<CapturedFrame>,
}
```

Implementáciu handlera zmeň na:

```rust
impl GraphicsCaptureApiHandler for Capture {
    type Flags = mpsc::Sender<CapturedFrame>;
    type Error = CaptureError;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self { sender: ctx.flags })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame<'_>,
        control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        let width = frame.width();
        let height = frame.height();

        let pixels = {
            let buffer = frame.buffer()?;
            let mut scratch = Vec::new();
            buffer.as_nopadding_buffer(&mut scratch).to_vec()
        };

        self.sender.send(CapturedFrame {
            width,
            height,
            pixels,
        })?;

        control.stop();
        Ok(())
    }
}
```

V `main` po výbere zdroja, pred `Settings::new`, vytvor `let (sender, receiver) = mpsc::channel::<CapturedFrame>();`. V `Settings::new` nechaj `ColorFormat::Rgba8`, posledný argument `()` nahraď `sender`. Po `Capture::start(settings)?;` pridaj:

```rust
let captured = receiver.try_recv()?;
println!(
    "Po skončení snímania vlastníme {} × {} pixelov, {} bajtov RGBA.",
    captured.width,
    captured.height,
    captured.pixels.len()
);
```

`try_recv` tu nečaká: po úspešnom callbacku už správa musí byť odoslaná. Ak zdroj skončil bez snímky, vrátime chybu namiesto neobmedzeného čakania. Zrušenie pickeru sa naďalej rieši skorým návratom pred začiatkom snímania.

- `Flags` je v tomto traite všeobecný typ vstupných údajov pre handler, nie nutne bitové príznaky. Posledný argument nastavení sa sprístupní cez `ctx.flags`. Tu týmto spôsobom odovzdávame sender.
- `Capture` už má stav: vlastní odosielateľa kanála. Receiver zostáva v `main`.
- `frame.buffer()` sprístupní pixelové dáta CPU cez staging textúru a mapovanie. Surové riadky môžu obsahovať zarovnávaciu výplň (padding), preto ich nesmieme automaticky považovať za tesné `width * 4` riadky.
- `as_nopadding_buffer(&mut scratch)` poskytne tesne uložené pixely; pri paddingu používa pomocný vektor. Ak padding chýba, môže vrátiť pohľad priamo na pôvodné dáta a `scratch` zostane prázdny. Preto neposielame samotný `scratch`.
- `.to_vec()` skutočne kopíruje bajty do vlastneného `Vec<u8>`. Toto je iné než `Arc::clone`, ktoré zdieľa existujúcu alokáciu. Ide o jednoduchú učebnú cestu; pri paddingu môže vzniknúť aj dodatočná kópia cez scratch.
- Blok `{ ... }` vracia posledný výraz. Pred odoslaním vlastnených pixelov opustíme rozsah pixelového bufferu a jeho požičaní.
- `send(CapturedFrame { ... })` presunie vlastníctvo správy vrátane vektora do kanála. Samotné odoslanie nevytvára ďalšiu kópiu pixelov.
- RGBA8 má štyri bajty na pixel. Pri 1920 × 1080 očakávame 8 294 400 bajtov jednej nekomprimovanej snímky bez paddingu. Nie je to PNG, MP4 ani komprimovaný `VideoBlock`.
- Surové snímky nebudeme hromadiť do päťminútového replay buffera. Tento krok vysvetľuje vlastníctvo; finálna cesta najprv komprimuje video a pre NVENC zvážime odovzdanie GPU textúr bez zbytočného čítania cez CPU.

Over cez `cargo run --example native_capture`: po výbere zdroja očakávame jedinú prijatú snímku a výpis vlastnených pixelov po skončení snímania. Kód bol pripravený podľa lokálne dostupných zdrojov `windows-capture` 2.0.1; nové zmeny v učebnom príklade používateľ ešte len doplní a overí.

API: [FrameBuffer](https://docs.rs/windows-capture/2.0.1/windows_capture/frame/struct.FrameBuffer.html).

## Vizuálne overenie snímky: PNG v RAM a uloženie na príkaz

V `examples/native_capture.rs` zakódujeme získané RGBA pixely ako PNG. Pre tento učebný krok ponecháme v správe surové pixely aj PNG, aby sme mohli porovnať ich veľkosť. Ide iba o jednu snímku, nie o návrh pamäťového buffera pre nekomprimované video.

Pridaj importy (existujúci `use std::sync::mpsc;` nechaj):

```rust
use std::{fs, io::{self, Write}, time::{SystemTime, UNIX_EPOCH}};
use replay_recorder::config::load_config;
use windows_capture::encoder::{ImageEncoder, ImageEncoderPixelFormat, ImageFormat};
```

Do `CapturedFrame` pridaj pole `png: Vec<u8>`. V `on_frame_arrived`, po vytvorení `pixels` a pred odoslaním správy, pridaj:

```rust
let png = ImageEncoder::new(
    ImageFormat::Png,
    ImageEncoderPixelFormat::Rgba8,
)?
.encode(&pixels, width, height)?;
```

Správu odošli vrátane nového poľa:

```rust
self.sender.send(CapturedFrame {
    width,
    height,
    pixels,
    png,
})?;
```

PNG kódujeme v callbacku, kde knižnica zabezpečuje aktívny kontext Windows Runtime. Lokálna implementácia `ImageEncoder` používa Windows `BitmapEncoder` a `InMemoryRandomAccessStream`; nevytvára obrazový súbor. Po návrate zo snímania už nepredpokladáme, že zostala aktívna inicializácia Windows Runtime na danom vlákne.

V `main`, za prijatie `captured` a existujúci výpis, pred posledné `Ok(())`, vlož:

```rust
println!("PNG v RAM: {} bajtov.", captured.png.len());
println!("Pre uloženie napíš s a stlač Enter. Iný vstup skončí bez uloženia.");

let mut answer = String::new();
io::stdin().read_line(&mut answer)?;

if answer.trim() != "s" {
    return Ok(());
}

let config = load_config().map_err(|error| error.to_string())?;
let directory = config.output_directory();
fs::create_dir_all(directory)?;

let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(io::Error::other)?
    .as_nanos();
let path = directory.join(format!("screenshot-{timestamp}.png"));

let mut temporary = tempfile::Builder::new()
    .prefix("screenshot-")
    .suffix(".part")
    .tempfile_in(directory)?;

temporary.write_all(&captured.png)?;
temporary.as_file().sync_all()?;
temporary.persist_noclobber(&path)
    .map_err(|error| error.error)?;

println!("Snímka uložená: {}", path.display());
```

- `ImageFormat::Png` určuje výstupný súborový formát, `ImageEncoderPixelFormat::Rgba8` usporiadanie vstupných pixelov. Musí zodpovedať `ColorFormat::Rgba8` v capture nastaveniach.
- `encode(&pixels, width, height)` požičiava vstupné bajty a vracia nový vlastnený `Vec<u8>` s celou PNG štruktúrou a komprimovaným obrazom. Premenovaním surových bajtov na `.png` by obrázok nevznikol.
- `&Vec<u8>` sa pri tomto volaní automaticky prevedie na požičaný rez `&[u8]`, ktorý API očakáva. Táto konverzia bajty nekopíruje.
- PNG je bezstratový obrazový formát. Veľkosť závisí od obsahu a nemusí byť vždy menšia než surové dáta. Zakódovanie jednej snímky nie je video enkódovanie s časovými závislosťami medzi snímkami.
- `read_line` tu blokuje až po skončení snímania, takže nezastavuje aktívny capture callback. Ukladáme skoršiu zachytenú snímku, nie novú snímku v okamihu zadania `s`.
- `load_config` vracia `Box<dyn Error>`, zatiaľ čo `main` príkladu používa `Box<dyn Error + Send + Sync>`. Učebný prevod cez `map_err(|error| error.to_string())?` zachová text a umožní vytvorenie požadovanej krabicovej chyby; stratí však konkrétny pôvodný typ chyby. Jednotné typy chýb môžeme neskôr upratať.
- Tento príklad používa z konfigurácie iba cieľový priečinok. Výstup je zámerne PNG bez ohľadu na nastavený video kontajner MP4/MKV.
- Bez potvrdenia sa pixely aj PNG pri odchode z `main` uvoľnia a nevytvorí sa súbor ani cieľový priečinok.

Ručné overenie cez `cargo run --example native_capture`: vyber zdroj, potvrď uloženie cez `s`, otvor výsledný PNG z vypísanej cesty a skontroluj obsah, rozmery a farby. Zopakuj bez potvrdenia a over, že ďalší obrázok nevznikol. Nový kód a živý výsledok má používateľ overiť po doplnení; postup bol pripravený podľa lokálneho `windows-capture` 2.0.1 `encoder.rs`.

## Menej map_err: zjednotenie návratových typov chýb

Pri extrakcii `prompt_saving` sa návratový typ zmenil na `Result<(), String>`. I/O, systémový čas aj tempfile však vracajú konkrétne chybové typy. Preto bolo potrebné opakovať `.map_err(|error| error.to_string())?`. Textový prevod stráca pôvodný konkrétny typ a prípadný reťazec príčin.

Najmenšia úprava pri existujúcom `type CaptureError = Box<dyn std::error::Error + Send + Sync>`:

```rust
// examples/native_capture.rs
fn prompt_saving(captured_frame: CapturedFrame) -> Result<(), CaptureError> {
    // ...
}

// src/config.rs, import std::error::Error už existuje
pub fn load_config() -> Result<Config, Box<dyn Error + Send + Sync>> {
    // Pôvodné telo s ? zostáva.
}
```

Potom môžeme v `prompt_saving` použiť priamo:

```rust
io::stdin().read_line(&mut answer)?;
let config = load_config()?;
fs::create_dir_all(directory)?;
let timestamp = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)?
    .as_nanos();
// ...
temporary.write_all(&captured_frame.png)?;
temporary.as_file().sync_all()?;
temporary.persist_noclobber(&path)?;
```

I/O chyba, `SystemTimeError` aj `tempfile::PersistError` môžu byť uložené do uvedeného boxu pri splnení `Error + Send + Sync + 'static`. Operátor `?` využije existujúce konverzie `From`; neprevádza tieto chyby na text. `PersistError` si pritom zachová vlastníctvo dočasného súboru až do svojho zrušenia alebo explicitného spracovania. Validácia konfigurácie zatiaľ stále vracia textovú chybu; aj pre `String` existuje osobitná konverzia do tohto boxu.

Samotná zmena `prompt_saving` nestačí: `Box<dyn Error>` z pôvodného loadera už nezaručuje `Send + Sync`, hoci konkrétna zabalená chyba tieto vlastnosti môže mať. Informáciu preto zachováme už v návratovom type loadera. `main` s `CaptureError` ostáva bez zmeny; pôvodný hlavný program s `Box<dyn Error>` dokáže silnejšie ohraničený box prijať so zahodením dodatočných vlastností.

Neskôr môžeme rovnaký alias pomenovať neutrálne `AppError` a sprístupniť z knižnice, namiesto opakovania typu. Ak bude program potrebovať rozlišovať druhy chýb cez `match`, bude vhodný vlastný enum (prípadne derive cez `thiserror`). Ak chceme prevažne pridávať kontext v aplikácii, ďalšou možnosťou je `anyhow`; pre toto zjednodušenie netreba pridávať závislosť.

`map_err` má naďalej zmysel, keď zámerne meníme typ chyby, pridávame kontext alebo prispôsobujeme hranicu API. Nemusí byť pri každom `?`.

Po ručnej úprave over: `cargo check --example native_capture` a `cargo check --bin replay-recorder`. Toto je odporúčaná úprava pri review, nie už vykonaná zmena zdrojového kódu.

## Krátke H.264 video v RAM

Celý príklad: [examples/native_video.rs](examples/native_video.rs).

Spustenie z koreňa projektu:

```powershell
cargo run --example native_video
```

Vyber plochu (alebo okno s párnymi rozmermi) a pohybuj jeho obsahom.
Po približne piatich sekundách sa nahrávanie dokončí. Kontrola času beží
v `on_frame_arrived`, takže pri statickom zdroji môže čakať na ďalšiu snímku.
Potom `s` + Enter uloží MP4 do `output_directory` z konfigurácie.
Iný vstup dáta zahodí. Over uložené video prehratím.

Tento samostatný experiment používa pevne H.264, MP4, 30 FPS v nastavení
enkodéra, bitrate 8 Mbit/s a vypnutý zvuk. Nastavenie FPS enkodéra samo
nevytvára nové snímky zo statického zdroja. Z konfigurácie zatiaľ používa
iba cieľovú zložku; nejde ešte o hlavný recorder s rotáciou blokov.

### Pamäťový stream namiesto súboru

`InMemoryRandomAccessStream` pochádza z priamej závislosti
`windows = { version = "0.62.2", features = ["Storage_Streams"] }`.
Verzia sa zhoduje so závislosťou použitej knižnice `windows-capture`.

```rust
let stream = InMemoryRandomAccessStream::new()?;
```

`VideoEncoder::new_from_stream(...)` zapisuje zakódované video do tohto
streamu. `send_frame(frame)` odovzdáva snímky enkodéru; nevytvárame z nich
PNG ani vlastný zoznam surových pixelov. Capture tu používa `Bgra8`.

`finish()` dokončí spracovanie a MP4 kontajner. Až potom prečítame stream
cez `DataReader` do `Vec<u8>` a pošleme vlastníctvo vektora cez kanál.
Počas čítania existuje stream aj jeho kópia vo vektore. Je to dočasná cena
tohto krátkeho pokusu; zatiaľ nepoužíva naše zdieľané `Arc<VideoBlock>`.
Zápis mediálnych dát do súboru sa začne až po potvrdení používateľom.

### Prečo `Option<VideoEncoder>` a `take()`?

Enkodér vytvárame pri prvej snímke, keď poznáme rozmery. Dovtedy je `None`.
Počas nahrávania si ho iba mutabilne požičiavame:

```rust
if let Some(encoder) = self.encoder.as_mut() {
    encoder.send_frame(frame)?;
}
```

`finish(self)` však potrebuje vlastníctvo enkodéra. Z požičaného `&mut self`
nemôžeme iba vysťahovať pole a ponechať ho neinicializované. `take()` nahradí
hodnotu za `None` a vráti pôvodný `Option`:

```rust
if let Some(encoder) = self.encoder.take() {
    encoder.finish()?;
}
```

Tu už `encoder` vlastníme a môžeme ho spotrebovať. `take()` nekopíruje
enkodér ani obsah videa.

Pamäťový stream s jedným dokončeným MP4 ešte nie je rotačný buffer.
Ďalší krok musí riešiť samostatné zakódované bloky, časovanie a kľúčové
snímky; nestačí odstrihávať ľubovoľné bajty z MP4.

## VideoSettingsSubType: kodeky a pixelové formáty

Enum v `windows-capture 2.0.1` obsahuje komprimované video aj surové
pixelové formáty. Prítomnosť variantu nezaručuje dostupný enkodér ani
kompatibilitu so zvoleným kontajnerom na konkrétnom počítači.

| Varianty | Význam |
| --- | --- |
| `H264` | AVC; naša úvodná voľba pre komprimované video. |
| `HEVC` | H.265; zvyčajne lepšia kompresia pri podobnej kvalite, závisí od enkodéra a obsahu. |
| `VP9` | Ďalší moderný kompresný formát; dostupnosť enkódovania treba overiť. |
| `H264ES`, `HEVCES` | Rovnaká rodina kodeku; ES znamená elementary stream, vzorky môžu obsahovať časti snímok alebo viac snímok. Nie vyššia kvalita. |
| `MJPG` | Motion JPEG: jednotlivé snímky komprimované ako JPEG. |
| `H263`, `MPEG1`, `MPEG2` | Staršie video kodeky. |
| `WMV3`, `WVC1` | Windows Media Video 9 a VC-1. |
| `RGB24`, `RGB32` | Surové RGB, 3 alebo 4 bajty na pixel; RGB32 nemá užitočnú alfu. |
| `ARGB32`, `BGRA8` | Surové 32-bitové farebné formáty s alfa kanálom. |
| `IYUV`, `YV12`, `NV12` | Surové YUV 4:2:0; odlišné rozloženie jasovej a farebných zložiek v pamäti. |
| `YUY2` | Surové YUV 4:2:2. |
| `L8`, `L16` | Iba jas, 8 alebo 16 bitov na pixel. |
| `D16` | 16-bitová hĺbka, nie bežný farebný obraz plochy. |

Naša cesta: `BGRA8 snímka -> H.264 enkódovanie -> MP4 kontajner`.
MP4 je obal pre video, zvuk a časovanie. H.264 je kompresný formát.
NVENC je implementácia enkodéra od NVIDIA, nie ďalší kontajner alebo
hodnota tohto enumu.

Pri rovnakom skutočnom priemernom bitrate a trvaní je veľkosť videa
približne rovnaká nezávisle od kodeku. Lepšia kompresia umožňuje znížiť
bitrate pri podobnej kvalite. Odhad RAM: `bitrate / 8 * sekundy` plus réžia.

Zdroje: [VideoEncodingProperties.Subtype](https://learn.microsoft.com/en-us/uwp/api/windows.media.mediaproperties.videoencodingproperties.subtype),
[Video Subtype GUIDs](https://learn.microsoft.com/en-us/windows/win32/medfound/video-subtype-guids).

## Trait a vymeniteľné implementácie enkodéra

Spustiteľný učebný príklad: [examples/encoder_trait.rs](examples/encoder_trait.rs).
Spusti `cargo run --example encoder_trait`.
Čísla zastupujú snímky; príklad nevykonáva kompresiu ani snímanie.

```rust
trait VideoEncoder {
    fn encode(&mut self, frame_id: u32) -> Option<EncodedPacket>;
    fn finish(&mut self) -> Option<EncodedPacket>;
}
```

`trait` definuje spoločné správanie. `impl VideoEncoder for DelayedEncoder`
implementuje toto správanie pre konkrétny typ. `ImmediateEncoder` vráti
paket hneď; `DelayedEncoder` si aktuálny paket ponechá a vráti predchádzajúci.
`None` znamená, že zatiaľ nemáme výstup; nie je to chyba.

```rust
fn record(encoder: &mut dyn VideoEncoder) {
    // Voláme metódy traitu bez znalosti konkrétnej implementácie.
}
```

`&mut` je mutabilné požičanie, nie presun vlastníctva. `dyn VideoEncoder`
označuje trait objekt: konkrétna implementácia metódy sa vyberie za behu.
Je to podobné volaniu virtuálnej metódy cez referenciu na rozhranie v C++.
Samotné `&mut dyn VideoEncoder` nepotrebuje alokáciu cez `Box`.

Oneskorená implementácia používa:

```rust
let previous = self.pending.take();
self.pending = Some(EncodedPacket { frame_id });
previous
```

`take()` presunie starý `Option` von a nechá `None`. Potom uložíme nový
paket. Posledný výraz bez bodkočiarky vráti predchádzajúcu hodnotu.
Pri vstupoch 1, 2, 3 dostaneme `None`, paket 1 a paket 2. `finish()`
vyberie zostávajúci paket 3. `#[derive(Default)]` v tomto príklade vytvorí
`DelayedEncoder` s poľom `pending: None`.

Pôvodné rozhranie vyššie bolo zámerne zjednodušené: nemá chyby, časovanie, skutočné
video ani asynchrónne preberanie viacerých paketov. Produkčné rozhranie
doladíme podľa prvej reálnej implementácie. Tu `finish(&mut self)` iba
vyprázdni simulovaný stav; na rozdiel od metódy v `windows-capture`
nespotrebúva vlastníctvo enkodéra.

## Enkodér: Result<Option<T>, E> a propagácia chyby

Príklad `examples/encoder_trait.rs` teraz rozširuje predchádzajúcu lekciu
o `Result<Option<EncodedPacket>, EncoderError>` pri `encode` aj `finish`.

| Hodnota | Význam |
| --- | --- |
| `Ok(Some(packet))` | Úspech s dostupným výstupom. |
| `Ok(None)` | Úspech, zatiaľ bez výstupu. |
| `Err(error)` | Zlyhanie. |

Chyba je vlastný enum:

```rust
#[derive(Debug)]
enum EncoderError {
    InvalidFrameId,
}
```

`Debug` umožní výpis cez `{error:?}`. Na použitie vlastného typu v
`Result` netreba implementovať `std::error::Error`; to doplníme, keď bude
potrebné jeho zapojenie do všeobecného spracovania chýb.

Číslo snímky 0 je neplatné iba podľa pravidla tejto simulácie.
`return Err(EncoderError::InvalidFrameId)` ukončí `encode`.
`encoder.encode(frame_id)?` v `record` pri chybe ukončí aj `record`.
Pri úspechu `?` rozbalí `Result` a ponechá `Option` pre následný `match`.

`record` prijíma `frames: &[u32]`, teda požičaný pohľad na čísla snímok.
`for &frame_id in frames` vyberá jednotlivé `u32` hodnoty kopírovaním;
`u32` implementuje `Copy`. Celý zoznam sa nekopíruje.

Pri vstupoch `[1, 0, 2]` sa spracuje 1 a na 0 sa vráti chyba do `main`,
kde ju vypíšeme cez `match`. Snímka 2 sa už nespracuje. Ani následné
`finish()` v `record` sa pri tomto predčasnom návrate nezavolá:
`?` nie je mechanizmus `finally`. Upratanie zdrojov cez `Drop` je odlišné
od úspešného dokončenia a získania zostávajúcich paketov.

Spustenie: `cargo run --example encoder_trait`. Overené sú normálny
okamžitý aj oneskorený výstup a cesta simulovanej chyby. Ide o učebnú
simuláciu; žiadne video sa nesníma ani neukladá.

## Výber enkodéra za behu: Box<dyn VideoEncoder>

Príklad `examples/encoder_trait.rs` teraz vyberá jednu implementáciu:

```rust
fn create_encoder(delayed: bool) -> Box<dyn VideoEncoder> {
    if delayed {
        Box::new(DelayedEncoder::default())
    } else {
        Box::new(ImmediateEncoder)
    }
}
```

Obe vetvy vracajú spoločný typ `Box<dyn VideoEncoder>`, hoci konkrétne
objekty majú odlišné typy. `Box` vlastní objekt a pri svojom zániku ho
zahodí. Pri bežnom type s nenulovou veľkosťou je objekt alokovaný na halde;
náš bezstavový `ImmediateEncoder` má nulovú veľkosť a alokáciu nepotrebuje.
`dyn` umožňuje volať implementáciu traitu konkrétneho objektu za behu.
Približná analógia v C++ je `std::unique_ptr<Interface>` s virtuálnymi metódami.
Na rozdiel od `Arc` tu nejde o zdieľané vlastníctvo.

```rust
let mut encoder = create_encoder(delayed);
record(encoder.as_mut(), &[1, 2, 3])?;
```

`as_mut()` na tomto Boxe poskytne `&mut dyn VideoEncoder`. Funkcia
`record` objekt iba požičiava, vlastníkom zostáva `encoder`. Nedochádza
ku kopírovaniu objektu ani k presunu vlastníctva do `record`.

Výber skúšame cez argument príkazového riadka:

```powershell
cargo run --example encoder_trait
cargo run --example encoder_trait -- --delayed
```

`--` oddeľuje argumenty Cargo od argumentov spúšťaného programu.
`std::env::args().any(|arg| arg == "--delayed")` zisťuje prítomnosť
prepínača; ide o jednoduchý učebný parser. Prvý príkaz vyberie okamžitý
enkodér a druhý oneskorený. Oba na konci stále spustia osobitnú simuláciu
chyby z predchádzajúcej lekcie.

V produkcii bude výber zisťovať schopnosti zariadenia a skúšať inicializáciu.
Tá môže zlyhať, preto bude vytvorenie vracať napríklad
`Result<Box<dyn VideoEncoder>, EncoderError>`. Tento príklad žiadne GPU
schopnosti nezisťuje.

## Kľúčové snímky a bezpečný začiatok replayu

Príklad: [examples/keyframes.rs](examples/keyframes.rs).
Spustenie: `cargo run --example keyframes`.

Komprimovaná snímka môže potrebovať skoršie snímky na dekódovanie.
Preto pri zahadzovaní starého obsahu potrebujeme zachovať bezpečný bod
začiatku dekódovania. V simulácii ho označujeme `K`; závislé snímky sú `P`.
Jedna snímka tu zámerne trvá jednu sekundu:

```text
Cas:    0 1 2 | 3 4 5 | 6 7 8 | koniec 9
Typ:    K P P | K P P | K P P |
```

Posledných päť sekúnd je interval `[4, 9)`. Začiatok na 4 by vynechal
potrebnú snímku na 3. Preto zachováme `[3, 9)`, teda šesť sekúnd.
Začiatok na ďalšej K na 6 by poskytol iba tri sekundy.
Zachovanie staršieho začiatku a presné orezanie výsledného prehrávania
sú odlišné úlohy; táto lekcia rieši len zachovanie potrebných dát.

```rust
packets.iter().rposition(|packet| {
    packet.is_keyframe && packet.timestamp <= desired_start
})
```

`rposition` hľadá od konca a vracia `Option<usize>`: index poslednej
vyhovujúcej položky alebo `None`. `let Some(index) = ... else { return; };`
rozbalí index alebo opustí funkciu. Výraz `&packets[index..]` vytvorí
požičaný slice od indexu do konca; nekopíruje dáta ani nevytvára vlastnený
snapshot. `saturating_sub` pri odčítaní väčšieho trvania vráti nulu.

Ide o simuláciu metadát: neobsahuje kompresiu, MP4 ani reálny replay.
Predpokladá rovnaké poradie zobrazenia a dekódovania a nezávislé skupiny.
V reálnej implementácii musíme z enkodéra získať bezpečné body začiatku,
časovanie a konfiguráciu kodeku (napr. SPS/PPS pri H.264). Nestačí ľubovoľnú
I-snímku bez overenia považovať za nezávislý začiatok.

Náš `ReplayBuffer` už vyhadzuje celé bloky. Neskôr zabezpečíme, aby hranice
týchto blokov rešpektovali závislosti komprimovaných snímok.

Zdroj: [Microsoft: CleanPoint](https://learn.microsoft.com/en-us/windows/win32/medfound/mfsampleextension-cleanpoint-attribute).

## Skladanie skupín snímok do ReplayBuffer

Spustenie učebného príkladu: `cargo run --example gop_buffer`.
Zdroj: [examples/gop_buffer.rs](examples/gop_buffer.rs).

Príklad používa existujúci `ReplayBuffer` a `VideoBlock`. Jeden bajt
v `data` predstavuje iba ID simulovanej snímky, nie komprimované video.
Každá snímka trvá jednu sekundu. Predpokladáme nezávislé skupiny začínajúce
K a zhodné poradie dekódovania a zobrazenia. Termín GOP znamená
Group of Pictures, skupina snímok; nie každý reálny GOP je nezávislý.

```text
Blok 0: K0 P1 P2
Blok 1: K3 P4 P5
Blok 2: K6 P7 P8
Aktívny: K9 ...
```

Rozpracovaný blok držíme ako `Option<VideoBlock>`. Pri novej K odovzdáme
predchádzajúci blok bufferu a založíme nový. Nová K patrí do nového bloku.
Nezastavujeme enkodér; uzavretie bloku je iba organizácia jeho výstupu.

```rust
if let Some(completed) = current.take() {
    buffer.push(completed)?;
}
```

`take()` nahradí hodnotu za `None`; pôvodný blok presunieme do bufferu.
Obsah jeho `Vec<u8>` sa nekopíruje. `current.as_mut()` umožní meniť
rozpracovaný blok cez požičanie; `current.as_ref()` ho iba čítať.

Po spracovaní snímok 0 až 9 zostávajú v päťsekundovom bufferi uzavreté
bloky `[3,4,5]` a `[6,7,8]` s trvaním spolu šesť sekúnd. Blok `[0,1,2]`
sa už odstránil. Aktívny blok `[9]` má ďalšiu sekundu a zostáva mimo bufferu.

Pôvodná verzia tohto príkladu ešte nerobila úplný snapshot do okamihu požiadavky:
`buffer.snapshot()` obsahuje iba uzavreté bloky, nie aktívny blok.
Pred reálnym ukladaním musíme vyriešiť zahrnutie už dostupných paketov
aktívnej skupiny bez zastavenia nahrávania. Bezpečne zdieľať aktívne dáta
vyžaduje vhodnú granularitu vlastníctva; uzavreté bloky už zdieľame cez Arc.
Skutočné pakety budú potrebovať aj hranice, časovanie a konfiguráciu kodeku;
samotné zlepenie ich bajtov nevytvorí MP4.

## Snapshot vrátane rozpracovaného bloku

Príklad `examples/gop_buffer.rs` teraz obsahuje `create_snapshot`:

```rust
fn create_snapshot(
    buffer: &ReplayBuffer,
    current: Option<&VideoBlock>,
) -> Vec<Arc<VideoBlock>> {
    let mut snapshot = buffer.snapshot();
    if let Some(block) = current {
        snapshot.push(Arc::new(VideoBlock {
            id: block.id,
            data: block.data.clone(),
            duration: block.duration,
        }));
    }
    snapshot
}
```

Uzavreté bloky zostávajú zdieľané cez `Arc` bez kopírovania ich bajtov.
Rozpracovaný blok kopírujeme, pretože pôvodný sa bude ďalej meniť.
`Vec<u8>::clone()` skopíruje obsah; `Arc::clone()` zdieľa pôvodný objekt.
V tomto kroku ide o úmyselnú cenu za jednoduchý stabilný snapshot.
Pre veľké aktívne bloky môže neskôr pomôcť zdieľanie nemenných paketov
cez `Arc` na úrovni jednotlivých paketov, nie iba celých skupín.

`current.as_ref()` z `Option<VideoBlock>` vytvorí `Option<&VideoBlock>`:
blok iba požičia a ponechá vlastníctvo v `current`. Snapshot vytvorí vlastnú
kópiu aktívneho bloku, takže po návrate nezostáva na tomto požičaní závislý.
`for block in &snapshot` zase prechádza snapshot požičaním a nespotrebuje ho.

Overenie cez `cargo run --example gop_buffer`:

```text
Snapshot: [3,4,5], [6,7,8], [9]
Ďalšie nahrávanie: aktívny blok [9,10]
Snapshot stále: [3,4,5], [6,7,8], [9]
```

Snapshot vzniká bez čakania na ďalšiu K. Zahŕňa len výstup už dostupný
z enkodéra; nevyťahuje jeho oneskorené pakety a nevolá `finish()`.
Príklad používa nezávislé skupiny a žiadne preusporiadanie snímok.
Reálny kodek potrebuje správne časovanie, konfiguráciu a dekódovateľný
koniec výstupu. Tento kód zatiaľ neukladá MP4 ani nespúšťa worker.

## ConfigHolder a riadený reload

Spustenie: `cargo run --example config_reload`.
Príklad používa `ConfigHolder` z `src/config.rs`, ktorý vlastní jednu platnú
konfiguráciu. Príkazy sa potvrdzujú Enterom: `s` vypíše aktuálny stav,
`r` načíta súbor znovu a `q` ukončí príklad. Tu `s` neukladá replay.

`get_config(&self) -> &Config` konfiguráciu iba požičiava. Zmena súboru
sama nezmení už načítaný objekt. `reload_config(&mut self)` najprv načíta,
parsuje a validuje novú konfiguráciu. Až pri úspechu nahradí starú:

```rust
let config = load_config()?;
self.config = config;
Ok(())
```

Pri chybe `?` vráti riadenie pred priradením, takže stará hodnota zostane.
Na úrovni aplikácie chybu reloadu zachytíme:

```rust
match holder.reload_config() {
    Ok(()) => println!("Nova konfiguracia nacitana."),
    Err(error) => eprintln!("Reload zlyhal, ponechavam povodnu: {error}"),
}
```

`ConfigHolder::new()?` pri prvom zlyhaní aplikáciu ukončí, pretože nemáme
žiadnu platnú konfiguráciu. Chybný reload cez `match` iba vypíšeme a ďalej
prijímame príkazy. Rozdiel je v rozhodnutí volajúceho, nie v type chyby.

Praktické overenie: uprav `duration_seconds`, ulož súbor a zadaj `s`;
hodnota zostáva pôvodná. Po `r` sa zmení. Nastav `fps = 0` a zadaj `r`;
uvidíš chybu a posledné platné hodnoty. Potom FPS oprav a znovu obnov.
Automaticky overené v dočasnej zložke: 300 s -> 120 s, neplatné FPS
neprepíše 120 s, proces pokračuje a skončí cez `q`.

Tento príklad aktualizuje iba konfiguráciu v pamäti. Nespúšťa capture,
nemení existujúci ReplayBuffer ani nereštartuje enkodér. Reload nastavení
a ich aplikovanie do bežiacich komponentov budú samostatné kroky.

## Windows tray: ikona, menu a event loop

Príklad: [examples/tray.rs](examples/tray.rs).
Spusti z koreňa projektu `cargo run --example tray`.
Pridané závislosti: `tray-icon 0.26.0` a `tao 0.37.1`.

Pri hodinách alebo pod šípkou skrytých ikon sa má zobraziť modrá ikona.
Pravý klik otvorí menu: zobrazenie konfigurácie v konzole, reload a ukončenie.
Tooltip výslovne uvádza, že ide o demo bez nahrávania. Konzolu ponechávame
na diagnostiku; príklad ešte nepoužíva farebné nastavenia tray z konfigurácie.
Zatvorenie konzoly môže ukončiť aj tento proces. Použi menu Ukončiť.

`tray-icon` spravuje ikonu a menu. `tao` poskytuje event loop, ktorý
spracováva systémové udalosti. Na Windows musí byť ikona vytvorená na
vlákne, ktoré obsluhuje príslušný event loop; tu je to hlavné vlákno.
Nie je potrebné vytvoriť viditeľné aplikačné okno.

```rust
let event_loop = EventLoopBuilder::<MenuEvent>::with_user_event().build();
let proxy = event_loop.create_proxy();
MenuEvent::set_event_handler(Some(move |event| {
    let _ = proxy.send_event(event);
}));
```

Kliknutie na položku vyvolá callback, ktorý odošle `MenuEvent` cez proxy.
Proxy zároveň prebudí event loop. `let _ =` tu úmyselne ignoruje chybu
odoslania pri zániku event loopu; nie je to všeobecný postup pre všetky chyby.

```rust
event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;
    // match event ...
});
```

`move` presunie použité hodnoty ako holder, menu a stav tray do closure.
Callback sa volá opakovane. `ControlFlow::Wait` čaká na udalosti bez
aktívneho pollingu; `ControlFlow::Exit` žiada ukončenie.
`*control_flow = ...` mení hodnotu cez mutabilnú referenciu.
Druhý parameter callbacku, označený `_`, v tomto príklade nevyužívame.

`Option<TrayIcon>` držíme v closure, aby ikona žila po celý beh.
Pri úvodnej udalosti `StartCause::Init` ju vytvoríme, pri Ukončiť zavoláme
`drop(tray.take())`, čím ju odstránime pred odchodom z event loopu.
Zlyhanie vytvorenia ikony vypíšeme a ukončíme proces s chybovým kódom.

`event.id` porovnávame s ID vytvorených položiek; nerozhodujeme podľa textu
menu. Reload používa existujúci ConfigHolder a pri chybe ponecháva starú
konfiguráciu. V tejto lekcii sa malý lokálny konfiguračný súbor číta priamo
v callbacku. Dlhé úlohy, najmä ukladanie replayu, patria do workera, aby menu
zostalo responzívne. `stdin.read_line()` do event loopu nepridávame.

Tray aplikácia beží v relácii prihláseného používateľa. Windows služba
je odlišný mechanizmus; pre tento používateľský recorder ju nepotrebujeme.
Štart po prihlásení a finálny beh bez konzoly vyriešime neskôr spolu
s viditeľným oznamovaním chýb a stabilnou cestou ku konfigurácii.

Manuálne overenie: zobrazenie ikony, pravý klik, výpis konfigurácie,
platný aj neplatný reload, Ukončiť a zmiznutie ikony. Kompilácia sama
nepotvrdzuje fungovanie ikon a kliknutí v konkrétnej Windows relácii.

Zdroj: [tray-icon, požiadavky na event loop](https://github.com/tauri-apps/tray-icon).

## Cvičenie: prepojenie TrayMenu s ConfigHolder

Cieľ: položky Show a Reload vykonajú skutočnú prácu s konfiguráciou.
Ide o postup na doplnenie používateľom, nie o už zapojenú funkcionalitu.

1. V `main` vytvor `ConfigHolder::new()?` a odovzdaj ho do
   `menu_tray.menu_loop(config)?`.
2. V `tray.rs` importuj `crate::config::ConfigHolder` a zmeň podpis na
   `pub fn menu_loop(self, mut config: ConfigHolder) -> Result<(), DefaultError>`.
3. V `Event::UserEvent` volaj
   `self.handle_event(event, control_flow, &mut config)`.
4. Metóde `handle_event` pridaj parameter `config: &mut ConfigHolder`.
   Show môže použiť `println!("{:#?}", config.get_config())`.
   Reload spracuj cez `match config.reload_config()`, aby chyba nezatvorila tray.

Vlastníctvo: main -> menu_loop -> move closure. `handle_event` si holder
pri každej udalosti iba mutabilne požičiava. `&self` sa nemení, pretože
nemeníme polia TrayMenu; meníme samostatne odovzdaný ConfigHolder.
`mut config` umožňuje požičať lokálnu hodnotu mutabilne, nepridáva zdieľanie.
Všetko sa zatiaľ vykonáva na jednom vlákne, takže netreba Arc ani Mutex.

`{:#?}` je prehľadný viacriadkový Debug výpis. Funguje vďaka
`#[derive(Debug)]` na konfiguračných typoch.

Overenie po doplnení: `cargo check --bin replay-recorder --offline`, potom
`cargo run --bin replay-recorder`. Show zobrazí aktuálnu konfiguráciu.
Po editácii súboru sa hodnoty zmenia až po Reload. Pri `fps = 0` reload
zlyhá a Show stále zobrazí poslednú platnú konfiguráciu. Potom FPS oprav.
Tento krok neprekonfiguruje recorder ani enkodér, iba načítaný ConfigHolder.

## Cvičenie: stav workera a opakované príkazy

Ďalší krok po zapojení tray a shutdown je evidovať simulovaný stav
priamo vo worker vlákne. Toto je postup na implementáciu používateľom.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecorderState {
    Idle,
    Recording,
}
```

Vo vnútri `thread::spawn(move || { ... })`, pred slučkou prijímania:
`let mut state = RecorderState::Idle;`.
Start mení Idle na Recording; Stop mení Recording na Idle.
Opakovaný Start alebo Stop nemení stav. Shutdown ukončí worker nezávisle
od stavu. V simulácii ide iba o enum, žiadne snímanie sa ešte nespúšťa.

`PartialEq` umožní `==`, `Debug` výpis cez `{:?}`. `Copy` dovolí
kopírovať tento malý enum pri priradení; neznamená zdieľanie dát.
Stav vlastní iba worker, preto zatiaľ nepotrebujeme Mutex.
Pri reálnom capture sa prechod na Recording uskutoční až po úspešnom
spustení a o potvrdenom stave sa dozvie UI cez spätnú udalosť.

V tray zatiaľ úspešné `send()` hlás ako odoslanie požiadavky, nie ako
potvrdené nahrávanie. Over poradie Start, Start, Stop, Stop, Start, Quit:
opakované príkazy nezmenia stav a Quit počká na worker cez shutdown/join.

## Cvičenie: worker oznamuje stav tray menu

Cieľ ďalšieho kroku: worker cez callback oznámi RecorderState; callback
pošle AppEvent do Tao event loopu. UI mení položky až po prijatí stavu.
Ide o návod, ktorý si používateľ zapája do src sám.

V tray.rs pridáme enum `AppEvent { Menu(MenuEvent), RecorderStateChanged(RecorderState) }`
a event loop vytvoríme ako `EventLoopBuilder::<AppEvent>`.
Menu callback posiela `AppEvent::Menu(event)`. Samostatné proxy zachytené
v callbacku workera posiela `AppEvent::RecorderStateChanged(state)`.

Konštruktor recorderu prijme
`notify: impl Fn(RecorderState) + Send + 'static`. `Fn` znamená opakovane
volateľný callback, `Send` umožní jeho presun do workera, `'static` znamená,
že nesmie obsahovať krátkodobé požičania (neznamená, že musí žiť navždy).
Worker po inicializácii oznámi Idle a po každej skutočnej zmene svojho
simulovaného stavu zavolá `notify(state)`.

Recorder vytvoríme v menu_loop po vytvorení event loopu a proxy.
V main už Recorder nevytvárame ani neposielame ako parameter.
Zachytený recorder sa naďalej ovláda cez kanál a shutdown/join.

V UI vetva RecorderStateChanged zavolá `set_enabled`: pri Idle povolí Start
a zakáže Stop, pri Recording opačne. Pred prvým potvrdením sú obe položky
zakázané. Už prijaté duplicitné príkazy naďalej ošetruje worker.
UI objekty zostávajú na hlavnom vlákne; do workera presúvame iba callback
s proxy. Žiadny polling ani mutex nie sú na túto komunikáciu potrebné.

Overenie po implementácii: pri štarte príde Idle, po Start príde Recording
a zmení sa dostupnosť položiek, po Stop príde Idle. Konzola a tooltip musia
stále jasne uvádzať simuláciu; skutočný capture nie je zapojený.

## Riadenie skutočného capture cez CaptureControl

Príklad: `cargo run --example capture_control`.
Sníma hlavný monitor cez `Monitor::primary()`, bez výberového dialógu.
Po prvej snímke a potom najviac približne raz za sekundu pri príchode
snímky vypíše počet snímok a rozmery. Po Enteri capture zastaví.
Pri statickom obsahu nemusia pribúdať snímky ani výpisy; pohybuj oknom.
Neukladá obraz, nekopíruje pixely do vlastného bufferu a neenkóduje video.

```rust
let control = Capture::start_free_threaded(settings)?;
// Volajúce vlákno môže ďalej spracovávať príkazy.
control.stop()?;
```

`Capture::start(settings)` blokuje volajúce vlákno až do ukončenia capture.
V príkazovej slučke workera by preto znemožnilo prijatie ďalšieho Stop.
`start_free_threaded` vytvorí vlákno spravované knižnicou a vráti riadiaci
objekt `CaptureControl<Capture, DefaultError>`. Prijímanie snímok prebieha
v callbackoch knižnice. `stop(self)` riadiaci objekt spotrebuje, požiada
o zastavenie a počká na capture vlákno; môže vrátiť chybu.

`type Flags = ()` a posledný parameter `()` znamenajú, že tomuto handleru
neposielame žiadne vlastné štartovacie dáta. `_ctx` a `_control` sú zámerne
nepoužité parametre. Na rozdiel od prvého screenshot príkladu nezastavujeme
snímanie zvnútra callbacku po prvej snímke.

V recorderi neskôr budeme držať
`Option<CaptureControl<Capture, DefaultError>>`: Start session vytvorí,
Stop ju cez take vyberie a zastaví, Shutdown ju zastaví pred ukončením workera.
Enter je iba ovládanie samostatného konzolového príkladu; do tray callbacku
čítanie stdin nepatrí. Pri integrácii treba spracovať aj samovoľné ukončenie
a chyby capture, aby UI neukazovalo neaktuálny stav.

Build príkladu bol overený. Príjem snímok a zastavenie vyžadujú manuálne
spustenie vo Windows relácii. Vytvorenie riadiaceho objektu samo nepotvrdzuje
príjem prvej snímky. `examples/` je v súčasnom .gitignore ignorované.

## Cvičenie: samovoľné ukončenie capture

Worker nesmie čakať navždy iba na používateľský príkaz, ak capture môže
skončiť na inom vlákne. V tomto učebnom kroku použijeme `recv_timeout`
s intervalom 100 ms a kontrolu `CaptureControl::is_finished()`.
Tento interval nesúvisí s FPS; spracovanie snímok beží na capture vlákne.

Zmeň `while let Ok(command) = receiver.recv()` na `loop`. Na začiatku
iterácie over `capture.as_ref().is_some_and(|c| c.is_finished())`.
Dokončenú session vyber cez `take()` a zavolaj `wait()`: pri úspechu
oznám Idle, pri chybe Error. `wait()` nežiada zastavenie, len vyzdvihne
výsledok vlákna. Preto ho voláme až po `is_finished()`.

Potom získaj príkaz:

```rust
let command = match receiver.recv_timeout(Duration::from_millis(100)) {
    Ok(command) => command,
    Err(mpsc::RecvTimeoutError::Timeout) => continue,
    Err(mpsc::RecvTimeoutError::Disconnected) => break,
};
```

Za tým zostane existujúci `match command`. Timeout nie je chyba capture,
iba neprítomnosť príkazu v danom intervale. Kanál prebudí recv_timeout
hneď pri príchode príkazu; nečaká vždy celých 100 ms.
Kontrola na začiatku každej iterácie funguje aj pri priebežných príkazoch.

Za celou slučkou zastav prípadnú zostávajúcu session cez take/stop,
aby sa upratalo aj pri odpojení všetkých odosielateľov. Existujúci Shutdown
môže session zastaviť už vo svojej vetve; následne je Option prázdny.

Manuálny test po implementácii: do on_frame_arrived dočasne pridaj
`if self.frames == 60 { return Err("Test capture failure".into()); }`.
Pri pohybe obrazu po 60 snímkach čakaj Error v tray bez kliknutia na Stop.
Pre čisté ukončenie namiesto Err použi vnútorné `capture_control.stop()`
a vráť Ok(()); čakaj Idle. Testovacie zásahy potom odstráň. Nepovažuj
neprichádzanie snímok zo statického obrazu za dôkaz ukončenia capture.

## Dohodnutý návrh recorderu

- Komprimované video uchovávame v RAM; aplikácia nebude vytvárať dočasné video súbory.
- Klávesová skratka uloží celý dostupný buffer, najviac nakonfigurovanú dĺžku.
- Bežne počítame s 5–10 minútami; pred naplnením sa uloží kratší dostupný záznam.
- Nahrávanie pokračuje aj pri ukladaní a uloženie buffer nevymaže.
- Pri 25 Mb/s má 5 minút približne 938 MB a 10 minút 1,875 GB komprimovaného videa, bez ďalšej réžie (desiatkové jednotky).
- Opakované požiadavky na uloženie budú potrebovať obmedzenie, aby nerástla spotreba RAM bez kontroly.
- Windows môže bežnú pamäť stránkovať; RAM buffer sám negarantuje nulové fyzické zápisy operačného systému.

Zatiaľ sa učíme na skúšobných bajtoch. Zachytávanie obrazovky, enkódovanie, časové hranice, zostavenie MP4/MKV, skratky a tray sú ďalšie kroky.

## Konfigurácia TOML

```toml
[recording]
enabled = true
duration_seconds = 300
fps = 60
bitrate_mbps = 25
output_directory = 'D:\Recordings'
format = "mp4"

[hotkeys]
save_replay = "Ctrl+Home"
```

Jednoduché úvodzovky v TOML umožňujú písať Windows cestu bez zdvojovania spätných lomiek. `PathBuf` sám nevytvára priečinok. MP4/MKV označujú kontajnery, nie kodeky.

Parsovanie overí štruktúru a typy. Naša validácia navyše kontroluje nenulovú dĺžku, FPS a bitrate a neprázdnu cestu. To ešte neoveruje podporu parametrov enkodérom ani oprávnenie zapisovať do priečinka.

Pozor: relatívna cesta `config.toml` sa vyhodnocuje voči aktuálnemu pracovnému adresáru procesu, nie automaticky voči umiestneniu executable. Pred automatickým štartom aplikácie vyriešime stabilné umiestnenie konfigurácie.
