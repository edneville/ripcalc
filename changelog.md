0.2.0

 * do ptr lookups as %p in format
 * use a cache for lookups

0.1.13

 * don't require input to be one IP per line
 * show subnets within network in format, suggested by andy@bitfolk.com
 * limit how much an encapsulating network can grow with --group

0.1.12

 * now builds on macos #1, thanks @sander_maijers
 * trim space around input
 * when using --inside or --outside if no input networks match, then exit 1

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

