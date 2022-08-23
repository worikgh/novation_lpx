#!/usr/bin/perl -w
use strict;
use lib(".");
use Daemon;

my $bin = `which yoshimi`;
chomp $bin;
-x $bin or die "Cannot find yoshimi";
my @cmd = ();

@cmd = ($bin,  '-i', '-J', '--alsa-midi=1', '-c', '-K', '-L', "'Simple Clonewheel.xiz'", '-N', 'Yoshimi01', '-R', '48000');
&run(@cmd);
sleep 1;

## Set the mode of the LPX to programmer mode
print `./lpx_mode 1`;
print `./lpx_mode 127`;

@cmd = ("./lpx_manager", "midi.cfg", 60, 1, 4, 6, 8, 11);
&run(@cmd);
