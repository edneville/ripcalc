0.1.12

 * now builds on macos #1, thanks @sander_maijers

0.1.11

 * new option --noexpand to not expand input networks in list context

0.1.10

 * When reading from file/stdin, --inside and --outside control printing of
   input addresses against lookup network. e.g. ripcalc --file - --inside
   192.168.0.0/16 only prints stdin addresses that are within 192.168.0.0/16
 * When --reverse is given, input/source/both can be assumed to be
   back-to-front
 * When stdin and args are given in --list context, both are processed
 * When --list is used and stdin has content, but --file - isn't present, it
   is assumed as wanted and will behave as if it was told to read from stdin
 * When --divide CIDR is used, the network will be divided into CIDR width
   subnets

