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

I resolved it with using `Performance` from `web_sys` to measure elapsed time, instead of `std::time::Instant`. The idea here is that in WASM we are not allowed underlying OS calls like, say, access to the timer, since WASM can't make assumptions about the host it's running on. So, we have to encode those assumptions ourselves and say, use, `web_sys` package to provide the missing OS functionality.

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

## Using conditionally compiled modules with cfg_if! macro

When I tried to customize my core library for using both with and without WASM, I had come up with a conditionally compiled module which I wanted to conditionally use, i.e. something like this:

```rust
#[cfg(feature = "for_wasm")]
pub mod web_duration {
    pub fn mk_duration { ... };
}

pub fn something() {
    if cfg!(feature = "for_wasm") {
        web_duration::mk_duration(...);
    } else {
        mk_duration(...);
    }
}
```

However, it failed with `unresolved module web_duration` when run without "for_wasm" feature. Apparently, `cfg!` macro just returns `true` or `false` in compile time, so effectively the compile code is equivalent to:

```rust
if false {
    web_duration::mk_duration(...)
} else {
    mk_duration(...)
};
```

However, this means that both side of the condition still have to be valid and compile fine. And `web_duration` is excluded from compilation by using `#[cfg(...)]`, so it doesn't work.

One solution to this is to use a crate `cfg_if` (which is quite popular!).
With that crate we can have something like this instead:

```rust
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "for_wasm")] {
        web_duration::mk_duration(...)
    } else {
        mk_duration(...)
    };
};
```

and it will work. `cfg_if` is just a clever declarative macro defined with `macro_rules!`. It collects all the statements and annotates them with explicit `#[cfg]` instead of just `cfg!`.

## React and long calls to WebAssembly.

In order to organize a game between a human and a bot, I needed to get moves alternatingly from the bot and from human. Getting move from human is easy in React: a move is basically a JS object produced in an event handler, i.e. user clicks a die and a card, JS event handler produces certain move object and then I apply that move object to the state, update the state and Reacts re-renders the view and updates DOM.

The question is: how and when do I need to request bot's move? I started with having an explicit button: "Get bot's move", you click that button, JS makes a call to WebAssembly, everything freezes for 3 seconds and then you get your move and then everything happens in the same way as this move has been received from human: the state is updated, UI is redrawn.

Then, as the next step, I've just added that long WebAssembly call after my components have been updated after user move, immediately asking for bot's move, which made sense. I put in `componentDidUpdate` React lifecycling method which seemed to be what I needed, since it happens after state has been updated and `render` has been called. So I expected that it will render user move, show it to me and then will wait for WebAssembly call return.

However, what was happening is that I didn't get the user move on the screen! Instead both moves were shown only after bot's reply! This puzzled me, since I explicitly saw that `render` after my user's move was called.

However, what's happening is:

1. `render` just draws some React element, which is something like virtual DOM
2. then it calculated the diff of that with the previous state to only update necessary parts
3. then it *continues* with `componentDidUpdate` call before actually returning, so the browser's rendering is not happening until React returns. That's why I was not seeing the human's move render before starting the long WebAssembly call.

In order to fix that I can either make the call asynchronous (by, say, wrapping it `setTimeout`) or using WebWorkers to run it in a separate thread. For now I've just used `setTimeout(getMoveFromTheBot, 10)` to give browser a chance to re-render.

### Helpful links:

- [My question on StackOverflow](https://stackoverflow.com/questions/62354011/react-doesnt-update-dom-during-render-when-there-is-a-long-running-function-i)
- [Another relevant SO question](https://stackoverflow.com/questions/41490837/reactjs-render-called-but-dom-not-updated)
- [Jake Archibald: "In The Loop"](https://www.youtube.com/watch?v=cCOL7MC4Pl0&t=685s), a nice talk with good visualizations and good jokes from the speaker
- ["What the heck is event loop anyway"](https://www.youtube.com/watch?v=8aGhZQkoFbQ) -- another video about event loop in the browser
- [SO question about React updating the DOM](https://stackoverflow.com/questions/57100042/when-and-in-what-order-does-react-update-the-dom)

# CSS

## `<fieldset>` can't be flex container in Chrome

See [Flexbug #9](https://github.com/philipwalton/flexbugs#flexbug-9) for more details.

## SVG images are scaled differently to raster images (like PNG or JPG)

So `object-position` does not work as expected for them and one can't use `object-position: center` to center the image.

See ["Hot to scale SVG" article from css-tricks](https://css-tricks.com/scale-svg) for details.

# Server configuration

## Serving at relative path

If you want to serve you app not in the root, but somewhere else (i.e. in my case it is served under `/rokumon`) I've just followed this guide:

https://create-react-app.dev/docs/deployment#building-for-relative-paths

Basically you have to set "homepage" property in `package.json` and basename property in `<BrowserRouter>` for client side routing, sweet!

Note that there were some other tutorials found on the web which proposed longer and more involved solutions, not sure why.

## MIME Types

It looks like serving WASM on a web server requires a little bit of MIME types configuration.
I use nginx, so in my case I had to add

```
types {
    application/wasm wasm;
}
```

immediately after `include mime.types;` stanza in the `nginx.config`.

Note that it's important to add it there (when I added it inside `server` context -- it was overriding all the types instead of appending to them).
