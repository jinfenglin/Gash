1. In this version of gash, error information will not be passed through piping. This feature could be find in bash, for example, you input "lsaaaaa | wc" you will get back "0 0 0".
2. The output for last command is wrapped in scoped thread.
3. There are a lot of duplication in code, which mainly result from the stdout is a handler which will stuck the loop.
4. 2 extra enum data type are included to annoate the piping and redirection state of the command.  
5. Gash support redirection in any place of chain of command lines, for example: 
 	cat < log | grep let | wc > output 
	cat log | wc < log2 | wc < log1
It will behave like what bash does.
6. Some command line may have abnormal behavior, for example:
	./zhttpto | wc 
in bash: terminal will wait the server to be closed then output the number of output lines
in gash it will output information of the first line from server. 
