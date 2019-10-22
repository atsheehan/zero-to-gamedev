# Game Loop Notes

**Framebuffer**: an array of bytes that represent pixel data to be rendered onto
the screen. 

**Double buffering**: Keep two framebuffers in memory and draw to the offscreen
buffer and regularly swap between them. Most low level API's will abstract these
concepts away and what's most important is to issue draw commands and then when
all of those are complete run whatever the native API's `swap` command is to
switch buffers.  

> Note: the `swap()` calls should be VERY fast otherwise it defeats the purpose
> of calling swap. Generally, you can think of a swap as just switching two
> pointers.

---

Game loop at a high level:

```
while (true) {
  processInput();
  update();
  render();
}
```

Jon Blow has a good point in that most of the perf related issues in game come
down to the last stage: rendering.

> Key aspect: run the game at consistent speed despite different hardware.

Spinning as fast as possible is a bad use of resources and some hardware won't
be able to keep up with a set frame rate like 60 fps. The solution is to use a
*fixed*** time step like so:

```
previous = getTime();
lag = 0.0;
while (true) {
  current = getTime();
  elasped = current - previous;
  previous = current;
  lag += elasped;
  processInput();

  while (lag >= MS_PER_UPDATE) {
    update();
    lag -= MS_PER_UPDATE;
  }
  render(lag / MS_PER_UPDATE);
}
```

## Resources

* [The Cherno on delta time](https://www.youtube.com/watch?v=pctGOMDW-HQ&t=633s)
* [Gaffer On Games: Fix Your Timestep](http://vodacek.zvb.cz/archiv/681.html)
