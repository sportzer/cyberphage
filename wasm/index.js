const { Game } = wasm_bindgen;

const WIDTH = 80;
const HEIGHT = 36;
const display = new ROT.Display();
display.setOptions({
    width: WIDTH,
    height: HEIGHT,
    fontSize: 18,
    fontFamily: "'Ubuntu Mono', monospace",
    bg: "silver",
});
document.body.appendChild(display.getContainer());

wasm_bindgen('./cyberphage_wasm_bg.wasm').then(() => {
    const seed = Math.floor(Math.random() * (1<<32));
    const game = Game.new(seed);
    game.set_size(WIDTH, HEIGHT);

    const renderLoop = () => {
        game.run();

        const toColor = (n) => ROT.Color.toHex([(n)&255, (n>>8)&255, (n>>16)&255]);
        for (let x = 0; x < WIDTH; x++) {
            for (let y = 0; y < HEIGHT; y++) {
                const ch = String.fromCharCode(game.get_ch(x, y));
                const fg = toColor(game.get_fg(x, y));
                const bg = toColor(game.get_bg(x, y));
                display.draw(x, y, ch, fg, bg);
            }
        }

        requestAnimationFrame(renderLoop);
    };

    document.addEventListener("keydown", e => {
        game.push_keydown_event(e.keyCode, e.ctrlKey, e.altKey, e.shiftKey);
        // TODO: figure out something which is less of a hack
        // (only block arrows here and space in keypress handler?)
        if(e.key.length != 1) {
            e.preventDefault();
        }
    });
    document.addEventListener("keypress", e => {
        game.push_keypress_event(e.charCode, e.ctrlKey, e.altKey);
        e.preventDefault();
    });
    display.getContainer().addEventListener("mousedown", e => {
        const pos = display.eventToPosition(e);
        game.push_mouse_press_event(pos[0], pos[1], e.button);
    });
    display.getContainer().addEventListener("mouseup", e => {
        const pos = display.eventToPosition(e);
        game.push_mouse_release_event(pos[0], pos[1], e.button);
    });

    let minDelta = 1e9;
    display.getContainer().addEventListener("wheel", e => {
        minDelta = Math.min(minDelta, Math.abs(e.deltaY));
        const pos = display.eventToPosition(e);
        game.push_mouse_wheel_event(pos[0], pos[1], Math.round(e.deltaY/minDelta));
        e.preventDefault();
    });

    requestAnimationFrame(renderLoop);
});
