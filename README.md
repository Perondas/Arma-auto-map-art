# Automatic Arma 3 map art


## Description
This application generates Auto Hot Key scripts from .svg files to draw
on the Arma 3 in game map. 

## Usage
First generate a .ahk file using the application.

Set the following flags:

##### Mandatory:
* -s \<SOURCE FILE\> A path to a .svg file

##### Optional:
* -d \<DESTINATION PATH\> The destination path. Default same as source
* -g \<GRAIN LEVEL\> Higher value makes image more jaggered. Default value 0.15
* -p \<PAUSE DURATION\> Time interval between individual lines. Prevents opening the marker dialogue. Default 750ms
* -c \<SCALE\> Scales the image. Default 1
* -i \<STARTING INTERVAL\> Time span between pressing the hot key and the drawing process beginning. Default 3000ms
* -x \<X OFFSET\> Offset on the x axis. Default 0px
* -y \<Y OFFSET\> Offset on the y axis. Default 0px

#### Flags:
* -f Filters out small grains from the file
* -o Slows down the drawing process to be more compatible with remote servers

I recommend running Arma 3 in borderless window. Run the generated script as administrator. Pressing the ESC key should immediately
stop the script. Press CTRL + b and tab back into Arma. This will draw a bounding box for the final drawing. Tab back out
and press CTRL + z, tab back in and let the script draw. If you don´t have enough time to tab back in, change the value in the
-i flag to something higher.

