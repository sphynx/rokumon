# Info

This is a web part of Rokumon, importing the WASM code compiled from
`rokumon_wasm`. It uses `webpack` JS bundler, I'm not sure why ;)
Perhaps because it was the default output in `wasm-pack` and was used in
the tutorial. I may potentially switch to `no-bundler` solution later.

Currently, the web page doesn't do much, it just creates a new game,
renders it as a String in Rust, passes that through WASM to JS and prints 
the game state on the web page.

Evetually it needs to be integrated with that React-based UI I have in
[another project](https://github.com/sphynx/rokumon-web), perhaps then
the bundler will come handy, we will see.

# Setup

0. Build stuff in `rokumon_wasm` (as specified [here](rokumon_wasm/README.md))

1. Install `npm` and `node.js`:

Go to its [page](https://www.npmjs.com/get-npm) to learn how.

2. Install our dependencies with `npm`:

```
npm install
```

It will download and install tons of JS stuff locally (seriously, ~600
packages!).

3. Start the web server with:

```
npm run serve
```

This will start `webpack-dev-server` locally and serve our page on `localhost`.

It should say something like this at the end:

```
ℹ ｢wds｣: Project is running at http://localhost:8080/
...
ℹ ｢wdm｣: Compiled successfully.
```

4. Go to `http://localhost:8080` in your browser and enjoy the power of Rust, WASM and JS combined! :D
