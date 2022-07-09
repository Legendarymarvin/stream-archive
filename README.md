# Stream Archive

## What is it?

A simple command line tool to record/archive livestreams as they are happening. It is meant to continuously run in the background, probably on a server, a Raspberry Pi or something of the sort. It will simply record all provided Twitch channels that are streaming while the program is running.

## How does it work?

Every 30 seconds it requests which of the provided channels are live (using the Twitch Helix API) and for those that are, it'll start a recording (using [streamlink](https://streamlink.github.io/)) in a new Thread. It also keeps track of which channels it is currently recording to not start redundant recordings every 30 seconds. The recordings are put into directories for its respective channels under a given directory or the run directory of the program. 

## Requirements

1. A [Twitch client-id and secret](https://dev.twitch.tv/console/apps/create) for authentication against Twitch Helix API.
2. Streamlink installed and in your PATH

## How to use

1. Copy config-example.json into config.json and fill in your client-id and secret.
2. List the channels you want to record in channels.txt (chess & chess24 are just examples and can of course be removed) one line per channel
3. Preferably use [screen](https://www.gnu.org/software/screen/) or something similar so you can exit the terminal without stopping the application
4. Start the application with `./stream-archive "/where/ever/you/want/your/recordings"`, if no directory is provided it will use the current directory.

## Couldn't this be done simpler and/or without the need for a client-id?

Yes! To both! One could do a simpler version of this, completely skip the Twitch api and just spam streamlink calls for every channel every 30 seconds. Streamlink even has an integrated command line argument to retry streams, but that will stop after it recorded successfully for the first time iirc. One could also replace the request to Helix with a simple GET Request to every single channel, but I am not a fan of spamming dozens of requests if I can use a single one.

I am aware that the need for a client-id will probably dissuade about 99% of potential users; maybe I will provide a version without a client-id in the future. But I also assume that tools for this use case already exist (probably even better ones!), I did this mostly for fun, not because I couldn't find an alternative. 

## Possible Improvements in the future, unordered

* Configurable naming scheme for files
* Sanitizing stream titles/categories to make sure they don't break filenames (did not have an issue *yet*)
* Maybe a little configuration TUI
* Option to rotate files, Stream recordings add up quick. Maybe some functionality to delete after x days or x TB filled
* Skipping reruns: As soon as I figure out how, the helix api provides a "type" value, but infuriatingly it only distinguishes between 'live' and 'error' (doesn't exist for not being live)

## Why is there no Windows version?

Honestly? Because the cross compilation failed and I didn't take the time to debug it. I think it should work on Windows, given streamlink is provided. I will probably try it at some point and provide a Windows release. But I assume most people will run this on some sort of Linux server anyway. 