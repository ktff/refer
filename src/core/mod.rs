mod collection;
mod container;
mod item;
mod key;
mod reference;
mod shell;

pub use collection::*;
pub use container::*;
pub use item::*;
pub use key::*;
pub use reference::*;
pub use shell::*;

/* NOTES

- Goal is to completely prevent memory errors, and to discourage logical errors.

- Nuzno je razdvojiti read step od write step zbog
bledanja side effecta izmjene from iz jedne mutacije u druge read/write funkcije.

- Ownership nema smisla kao feature za kolekcija. Ownership postoji za strukture koje
item normalno ima kao plain object, sve ostalo sto posotji kao item u kolekciji moze
biti referencirano od strane drugih te posto se nemoze removati ako postoji referenca
te da se nemoze sigurno izvesti mutacija ownera i onwed u isto vrijeme, ownership se
ne cini korisnim.

- Build on top:
   X. Polymorphism
   X. Chunked collections as one opaque collection derived from composite collection
        - Na konacnom kolekcijom je da napravi ovo
        - Moguce je pomoci s macro koji bi napravio chunked collection ze jedan tip
   x. Composite collections
        - Na konacnom kolekcijom je da napravi ovo
   x. Conncurrency
      - Kroz chunked collections se cini kao dobar put
      - Na korisniku je da doda lock na koleckije podatke ovisno o tome što joj treba.
   x. Key<Level> kako bi se razlikovalo kljuceve na razlicitim razinama.
      - Izvedeno pomocu Global/Local tipova
   x. Prostorni collection
      - Na korisnicima je da dodaju extra funkcije
   x. Reference lifetime
   x. Key<Locality>, no, it's the job of reference to track this.
   x. Item i Ostalo su odvojene stvari
      - Ovo je prakticno rjesenje za problem duplog MutEntry jedan posudenog od drugog nad istim itemom gdje unutrasnji
        MutEntry remova item te se vrati na vanjski MutEntry koji ocekuje valjani podatak ali ga nema.
      - Da se ovo fixa a da se omoguci followanje referenci,
   x. Izdvoji parrallelnost
   x. Ukloni Locality
   x. Ukloniti Path
   x. poly
   *. Composite NOTE: Nope. MOra biti flat struktura gdje su samo leaf itemi koji se mogu referencirati.
   *. unsized T | Wandrer
   *. [u8]      |
   ?. Delayed, Split Item Access, zahtjeva da se dropa potpora za locking

   Chunking can be done according to one of two points:
        a) Connection
        b) Data
*/
