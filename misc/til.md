# Issues I had and things I learned

This probably needs to be expanded and converted into a blog post or series of notes.
For now, it will be rather sketchy just to keep track of interesting issues which gave me some trouble.

## `rand` failed in runtime

Even though it compiles fine with `wasm32-unknown-unknown` target, it fails at runtime. Note that you'll need to get `console_error_panic_hook` crate to see the Rust panics in the browser, otherwise you'll just see something like "unreachable executed".

It's possible to compile `rand` with better wasm-support by specifying `wasm-bindgen` feature.

This actually teaches us that even though your crate might compile fine for wasm32 target, it can still fail in runtime missing some core functionality (that was actually quite surprising to me).

## Serialize enums with payloads

This is not supported in wasm-bindgen (you can only serialize C-style enums), so you can't just add [wasm-bindgen] to any enum and expect it to work by default.

Solutions:

1. use serde and serialise everything to JSON, which can be easily passed to/from JS and read there. 
2. redesign a new system of data types which are WASM friendly and convert from your normal datatypes (this looks laborious, but will probably make more sense if performance matters).

## Using pointers to structures

It was not clear how to pass a pointer to certain large structure (like, say, entire game) to JS. I wanted this to follow the normal conventions of keeping large structures in Wasm linear memory and only passing handles to JS, so that it can do something with it.

It looked from the tutorial that I just need to expose constructor like this:

```rust
[wasm-bindgen]
pub fn new(opts: Opts) -> SomeType {
    ...
}
```

to get that handle in JS. But I though that it requires SomeType and all its fields to be [wasm-bindgen] and therefore no enums, Vecs of arbitrary types and so on.

However, it has turned out that everything is good, we just need not to use `pub` on the fields of the struct which should not be directly serialized, so this works:

```rust

[wasm-bindgen]
pub struct SerializableStruct {
    pub f: u32,
}

// Note the lack of [wasm-bindgen].
// We don't need it here.
pub struct NonSerializableStruct {
    data: SomethingStrange, // which can't be annotated with [wasm-bindgen]
}

[wasm-bindgen]
pub struct Large {
    ns: NonSerializableStruct, // Note lack of `pub`
    pub s: SerializableStruct,
}

[wasm-bindgen]
impl Large {
    pub fn new() -> Self {
        ...
    }

    pub fn do_something_with_me(&mut self) {
        ...
    }
}
```

And then in JS we can just do something like this:

```js
import { Large } from 'wasm_package'

let large = Large.new();
large.do_something_with_me();
```

## `time` failed in run time

When I used bot's `with_duration`.
Still have to resolve that.

## Reconcile React with WebAseembly

In particular application done with `create-react-app`.

This post was very helpful:
https://www.telerik.com/blogs/using-webassembly-with-react

## Position elements with position:absolute

When we position something with `position:absolute` in CSS, the containing block is not just the parent, but actually something which was *positioned*, i.e. something which has `position` itself. It was tricky and non-trivial to understand from reading the docs on MDN.

So to solve this we can use `position:relative` on the parent, which does basically nothing except establishing that parent as container against which `top` and `left` are measured for children with `position:absolute`.

https://developer.mozilla.org/en-US/docs/Web/CSS/Containing_Block
https://developer.mozilla.org/en-US/docs/Web/CSS/position
https://www.freecodecamp.org/news/how-to-understand-css-position-absolute-once-and-for-all-b71ca10cd3fd/

## Serializing maps with non-String type with `serde_json`

Apparently this is not supported by design, since JSON only allows strings as object keys.

In order to handle this we need to write a custom serializer.

https://github.com/serde-rs/json/issues/402
https://github.com/serde-rs/serde/issues/1428
(and of course Serde documentation)

I ended up using a module and `#[serde(with = "serde_card")]` like this:

```rust
type Cards = BTreeMap<Coord, Card>;

#[derive(Serialize, Deserialize))]
pub struct Board {
    #[serde(with = "serde_cards")]
    pub cards: Cards,

    // ... other fields
}

mod serde_cards {
    use super::*;

    pub fn serialize<S>(cards: &Cards, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.collect_seq(cards)
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Cards, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = <Vec<(Coord, Card)>>::deserialize(d)?;
        let map = vec.into_iter().collect();
        Ok(map)
    }
}
```


