# nano project manager 
a simple and small CLI for managing media projects of any size!
## cool and epic icon !!!!
![alt text](https://github.com/kaweepatinn1/nanopm/blob/main/assets/icon_64x64.png?raw=true)
## installation
download the installer from releases and run it. that's it! you can now start using nanopm :)

## usage
Usage:

    nanopm [OPERATION] [ARGUMENTS]
-----------------------------------------------------------------------------------------------------------------
Operations: 

    new, n      | Initialize a new project in the current directory, creating a new config file from 
                  provided arguments, using defaults where missing.
    update, u   | Update the current config file based on provided arguments. Project Manager must already 
                  have been initialized.
    query, q    | Query the current project based on provided arguments. Project Manager must already have 
                  been initialized.
-----------------------------------------------------------------------------------------------------------------
Arguments: 
          
    CONFIG ARGS | Works with either new/update operations:
    
        -n, --name <String>             | Names the project and its directory. When used with update, uses 
                                          the old config file to rename the old directory to the new name.
        -dn, --deadname <String>        | Looks for a directory with this name, updating it with the new 
                                          name provided if it exists, using it as the new project directory.
        -d, --days <Integer>            | Sets the amount of footage days the project should account for.
        -c, --cameras <Integer>         | Sets the amount of cameras the project should account for.
        -s, --sound-sources <Integer>   | Sets the amount of sound sources the project should account for.
        -cl, --clean                    | Cleans the project folder after initializing, deleting all empty 
                                          folders not defined by the program.

    QUERY ARGS | You can use ONE type of query at a time. Works with query operations only:
       
        GENERAL QUERY:

            -g, --general               | Creates a general query of various important project folders. 
                                          Edit the list in config. Can return sorted by size.
                -ss, --sort-size        | Sorts general query by size. Must be used after a general query.
                -sd, --sort-default     | Sorts general query by... its default order... Kind of redundant. 
                                          Must be used after a general query.
    
        PARTIAL QUERY:
    
            -r, --root                  | Queries the full project directory, as well as returning project 
                                          config values.
            -d, --days                  | Queries each day in RUSHES.
            -c, --cameras               | Queries each camera. Combines all days into one entry for each 
                                          camera, displays each day separately if --unique is used.
            -s, --sound-sources         | Queries each sound source. Combines all days into one entry for 
                                          each source, displays each day separately if --unique is used.
            -u, --unique                | Stops nanopm from combining all days into one entry for --cameras 
                                          and --sound-sources. Unique folders are queried individually.
    
        FOLDER QUERY: 
    
            -f, --folder <String>       | Queries all folders with the name of the string. Can chain 
                                          multiple --folder calls to query multiple folder names at once.
    
        UNIVERSAL QUERY ARGS:
        -w, --write <String>            | Writes query result to file with the specified string path. 
                                          Uses timestamp for path instead if last parameter.
        -t, --timestamp                 | Adds a timestamp to the top of the query file, if written. 
                                          Does nothing if write is not specified. Sick!
        -q, --quiet                     | Does not log missing folder errors into the console.

## build instructions
since i currently dont have access to mac/linu you'll have to build this yourself if you use either of those os' :<
<br>guide will be quite beginner friendly because not every film/media nerd is also a computer nerd

#### step 1: rust UP!!!
install rust!! i recommend [rustup](https://rustup.rs/) it is very cool and awesome it helped me install rust very easily. just follow the steps provided and you'll be okay :)

#### step 2: clone this repo
download and install [git](https://git-scm.com/downloads).
once you have git, open your console and cd (change directory) into a folder you want to bulid this in. now use the

    git clone https://github.com/kaweepatinn1/nanopm

command to clone this repository.

#### step 3: cargo build!!!
run 

    cargo build 

in the same location. <br><br>

thats it! you can now find the .exe file in ./target/debug/nanopm.exe ! next step is possibly finding out how to add it to your system yourself because i have no idea how to do that on other operating systems, or just stick with running the .exe through console instead.