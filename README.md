# async looper thing
this is a little rust project for making a little looper!

it currently has 4 channels and is controlled by an [open stage control](https://openstagecontrol.ammd.net/) UI because I like OSC (both open stage control and open sound control) :)

# osc protocol

```
everything is prefaced with /{track # starting from 1}
send to port 2322
/feedback f f -> first param is unused, second one is feedback (0 - 1)
/input f f -> first param  is unused, second one is input volume (0 - 1)
/output f f -> first param  is unused, second one is output volume (0 - 1)
/pan f f -> first param  is unused, second one is pan (-1 - 1)
/record f f -> first param is unused, second one is 1 for record on, 0 for record off
/clear f f -> both params are unused, just clears the loop buffer :3
```

in the future there will be fun ui things
