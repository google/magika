// Custom rules shared by the CLI, Python and C tests. The built-in PNG rules are not enforced, so
// with these rules alone PNG files are identified by rules instead of the model.
rule custom_png
{
    meta:
        label = "png"
        enforced = true
        class = "full"
        fp_rate = 0
        fn_rate = 0
    strings:
        $signature = { 89 50 4E 47 0D 0A 1A 0A }
    condition:
        $signature at 0
}
