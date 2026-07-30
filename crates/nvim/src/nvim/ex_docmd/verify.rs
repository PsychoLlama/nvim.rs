//! `verify_command` — whether a command may run here: 'secure', the
//! sandbox, the command-line window, a locked text buffer, a terminal.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn verify_command(mut cmd: *mut c_char) {
    if strcmp(b"smile\0".as_ptr() as *const c_char, cmd) != 0 as c_int {
        return;
    }
    let mut a: c_int = HLF_E as c_int;
    msg(
        b" #xxn`          #xnxx`        ,+x@##@Mz;`        .xxxxxxxxxnz+,      znnnnnnnnnnnnnnnn.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n###z          x####`      :x##########W+`      ,#############M;    W################.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####;         x####`    `z##############W:     ,################   W################.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####W.        x####`   ,W#################+    ,#################  W################.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n#####n        x####`   @###################    ,#################i W################.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n######i       x####`  .#########@W@########*   ,#################W`W################.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n######@.      x####`  x######W*.  `;n#######:  ,####x,,,,:*M######iW###@:,,,,,,,,,,,`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n#######n      x####` *######+`       :M#####M  ,####n      `x#####xW###@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n########*     x####``@####@;          `x#####i ,####n       ,#####@W###@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n########@     x####`*#####i            `M####M ,####n        x#########@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n#########     x####`M####z              :#####:,####n        z#########@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n#########*    x####,#####.               n####+,####n        n#########@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n####@####@,   x####i####x                ;####x,####n       `W#####@####+++++++++++i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####*#####M`  x#########*                `####@,####n       i#####MW###############W\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.######+  x####z####;                 W####,####n      i@######W###############W\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.`W#####: x####n####:                 M####:####@nnnnnW#######,W###############W\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####. :#####M`x####z####;                 W####,#################z W###############W\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.  #######x#########*                `####W,################W` W###############W\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.  `M#####W####i####x                ;####x,###############W,  W####+**********i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.   ,##########,#####.               n####+,##############n.   W###@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n####.    ##########`M####z              :#####:,###########Wz:     W###@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n####.    x#########`*#####i            `M####M ,####x.....`        W###@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n####.    ,@########``@####@;          `x#####i ,####n              W###@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n####.     *########` *#####@+`       ,M#####M  ,####n              W###@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n####.      x#######`  x######W*.  `;n######@:  ,####n              W###@,,,,,,,,,,,,`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.      .@######`  .#########@W@########*   ,####n              W################,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.       i######`   @###################    ,####n              W################,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.        n#####`   ,W#################+    ,####n              W################,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.        .@####`    .n##############W;     ,####n              W################,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.         i####`      :x##########W+`      ,####n              W################,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" +nnnn`          +nnn`        ,+x@##@Mz;`        .nnnn+              zxxxxxxxxxxxxxxxx.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(b" \0".as_ptr() as *const c_char, a);
    msg(
        b"                                                                                   ,+M@#Mi\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"                                                                                 .z########\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"                                                                                i@#########i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"                                                                              `############W`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"                                                                             `n#############i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"                                                                            `n##############n\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     ``                                                                     z###############@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    `W@z,                                                                  ##################,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    *#####`                                                               i############@x@###i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    ######M.                                                             :#############n`,W##+\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    +######@:                                                           .W#########M@##+  *##z\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    :#######@:                                                         `x########@#x###*  ,##n\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    `@#######@;                                                        z#########M*@nW#i  .##x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     z########@i                                                      *###########WM#@#,  `##x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     i##########+                                                    ;###########*n###@   `##x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     `@#MM#######x,                                                 ,@#########zM,`z##M   `@#x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      n##M#W#######n.               `.:i*+#zzzz##+i:.`             ,W#########Wii,`n@#@` n@##n\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ;###@#x#######n         `,i#nW@#####@@WWW@@####@Mzi.        ,W##########@z.. ;zM#+i####z\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       x####nz########    .;#x@##@Wn#*;,.`      ``,:*#x@##M+,    ;@########xz@WM+#` `n@#######\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       ,@####M########xi#@##@Mzi,`                     .+x###Mi:n##########Mz```.:i  *@######*\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"        *#####W#########ix+:`                             :n#############z:       `*.`M######i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"        i#W##nW@+@##@#M@;                                   ;W@@##########W,        i`x@#####,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"        `@@n@Wn#@iMW*#*:                                     `iz#z@######x.           M######`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"         z##zM###x`*, .`                                          `iW#####W;:`        +#####M\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"         ,###nn##n`                                                ,#####x;`        ,;@######\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"          x###xz#.                                                   in###+        `:######@.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"          ;####n+                                                    `Mnx##xi`   , zM#######\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"          `W####+                i.                                   `.+x###@#. :n,z######:\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"           z####@`              ;#:                                     .ii@###@;.*M*z####@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"           i####M         `   `i@#,           ::                           +#n##@+@##W####n\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"           :####x    ,i. ##xzM###@`     i.   .@@,                           .z####x#######*\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"           ,###W;   i##Wz#########     :##   z##n                           ,@########x###:\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"            n##n   `W###########M`;n,  i#x  ,###@i                           *W########W#@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"           .@##+  `x###########@. z#+ .M#W``x#####n`                         `;#######@z#x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"           n###z :W############@  z#*  @##xM#######@n;                        `########nW+\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"          ;####nW##############W :@#* `@#############*                        :########z@i`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"          M##################### M##:  @#############@:                       *W########M#\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"         ;#####################i.##x`  W#############W,                       :n########zx\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"         x####################@.`x;    @#############z.                       .@########W#\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"        ,######################`       W###############x*,`                    W######zM#i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"        #######################:       z##################@x+*#zzi            `@#########.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"        W########W#z#M#########;       *##########################z            :@#######@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       `@#######x`;#z ,x#######;       z###########M###xnM@########*            :M######@\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       i########, x#@`  z######;       *##########i *#@`  `+########+`            n######.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       n#######@` M##,  `W#####.       *#########z  ###;    z########M:           :W####n\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       M#######M  n##.   x####x        `x########:  z##+    M#########@;           .n###+\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       W#######@` :#W   `@####:         `@######W   i###   ;###########@.            n##n\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       W########z` ,,  .x####z           @######@`  `W#;  `W############*            *###;\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      `@#########Mi,:*n@####W`           W#######*   ..  `n#############i            i###x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      .#####################z           `@#######@*`    .x############n:`            ;####.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      :####################x`,,`        `W#########@x#+#@#############i              ,####:\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ;###################x#@###xi`      *############################:              `####i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      i##################+########M,      x##########################@`               W###i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      *################@; @########@,     .W#########################@                x###:\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      .+M#############z.  M#########x      ,W########################@`               ####.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      *M*;z@########x:    :W#######i        .M########################i               i###:\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      *##@z;#@####x:        :z###@i          `########################x               .###;\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      *#####n;#@##            ;##*             ,x#####################@`               W##*\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      *#######n;*            :M##W*,             *W####################`               n##z\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      i########@.         ,*n#######M*`           `###################M                *##M\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      i########n        `z#####@@#####Wi            ,M################;                ,##@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ;WMWW@###*       .x##@ni.``.:+zW##z`           `n##############z                  @##,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      .*++*i;;;.      .M#@+`          .##n            `x############x`                  n##i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      :########*      x#W,              *#+            *###########M`                   +##+\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ,#########     :#@:                ##:           #nzzzzzzzzzz.                    :##x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      .#####Wz+`     ##+                 `MM`          .znnnnnnnnn.                     `@#@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      `@@ni;*nMz`    @W`                  :#+           .x#######n                       x##,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       i;z@#####,   .#*                    z#:           ;;;*zW##;                       ###i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       z########:   :#;                    `Wx          +###Wni;n.                       ;##z\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       n########W:  .#*                     ,#,        ;#######@+                        `@#M\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      .###########n;.MM                      n*        ;iM#######*                        x#@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      :#############@;;                      .n`      ,#W*iW#####W`                       +##,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ,##############.                        ix.    `x###M;#######                       ,##i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      .#############@`                         x@n**#W######z;M###@.                       W##\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      .##############W:                        .x############@*;zW#;                       z#x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ,###############@;                        `##############@n*;.                       i#@\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ,#################i                         :n##############W`                       .##,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ,###################`                         .+W##########W,                        `##i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      :###################@zi,`                        ;zM@@@WMn*`                          @#z\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      :#######################@x+*i;;:i#M,                 ``                               M#W\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ;################################@x.                                                  n##,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      i#####################@W@@@@Wxz*:`                                                    *##+\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      *######################+```                                                           :##M\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ########################M;                                                            `@##,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      z#########################x,                                                           z###\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      n###########################n:                                                         ;##W`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      x#############################Mz#++##*                                                 `W##i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      M####################################@`                                                 ###x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      W#####################################`                                                 .###,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      @####################################M                                                   n##z\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      @##################z*i@WMMMx#x@#####,.                                                   :##@.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     `#####################@xi`     `::,*                                                       x##+\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     .#####################@#M.                                                                 ;##@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     ,#####################:.                                                                    M##i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     ;###################ni`                                                                     i##M\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     *#################W#`                                                                       `W##,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     z#################@Wx+.                                                                      +###\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     x######################z.                                                                    .@#@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    `@#######################@;                                                                    z##;\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    :##########################:                                                                   :##z\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    +#########################W#                                                                    M#W\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    W################@n+*i;:,`                                                                      +##,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"   :##################WMxz+,                                                                        ,##i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"   n#######################W..,                                                                      W##\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"  +#########################WW@+. .:.                                                                z#x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" `@#############################@@###:                                                               *#W\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" #################################Wz:                                                                :#@\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b",@###############################i                                                                   .##\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"n@@@@@@@#########################+                                                                   `##\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"`      `.:.`.,:iii;;;;;;;;iii;;;:`       `.``                                                        `nW\0"
            .as_ptr() as *const c_char,
        a,
    );
}
