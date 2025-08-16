# Pre-build command

Sometimes a command needs to be run before running ```docker build```, you can configure this in the *pre build* section. 

There are two fields to pass for *pre build*. the first is *path*, which changes the working directory. To run the command in the root of the repo, just pass ```.```. The second field is *command*, this is the shell command to be executed after the repo is cloned.

For example, say your repo had a folder in it called ```scripts``` with a shell script ```on-clone.sh```. You would give *path* as ```scripts``` and command as ```sh on-clone.sh```. Or you could make *path* just ```.``` and then the command would be ```sh scripts/on-clone.sh```. Either way works fine.