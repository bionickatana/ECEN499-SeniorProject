# Quickstart

prerequisites: Rake installed on your system.

Run `rake setup` to download the needed SDK libraries. It may also require you
to install compiler tools as well.

Run `rake build` to build the project. This will result in a binary that you
can flash to the device either through the 'drag-and-drop' or by putting the
device in bootselect mode and running `rake flash`.

The Rake tasks are structured so that you can actually just run `rake flash`
and if the binary needs to be rebuilt it will be


# Slowstart

Configuring of pins is done in the `CMakeLists.txt` file. This is because some
of the libraries that we use are built into the C++ SDK for the RP2xxx and
require the environment variables in the `CMakeLists.txt file`. Rather than
having multiple places for configuration we opted to place it all there even
though it is a little unconventional.
