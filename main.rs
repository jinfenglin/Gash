//
// gash.rs
//
// Starting code for PA2
// Running on Rust 1.0.0 - build 02-21
//
// Brandeis University - cs146a - Spring 2015

extern crate getopts;

use getopts::{optopt, getopts};
use std::old_io::BufferedReader;
use std::process::{Command, Stdio};
use std::old_io::stdin;
use std::thread;
use std::{old_io, os, old_path,path};
use std::str;
use std::io::{Write,Read};
use std::old_io::File;
use std::sync::mpsc::{channel, Sender, Receiver};
struct Shell<'a> {
    cmd_prompt: &'a str
}
const MSG_S: usize = 128;
//define the data structure for transmiting inside pipe
struct pack
{
    content: [u8;MSG_S],
    size: usize,
    end_flag: bool
}
//the rediection status for every command
enum redirection_sign{
    no,input,output
}
//the pipe status for every command
enum pipe_sign{
    pipe_in,pipe_out,pipe_both,pipe_none
}
//defining shell methods 
impl <'a>Shell<'a> {
    fn new(prompt_str: &'a str) -> Shell<'a> {
        Shell { cmd_prompt: prompt_str }
    }


    fn cd(&self,dir_path: &str){
        match os::change_dir(&Path::new(dir_path)){
            Ok(v)  => (),
            Err(e) => {println!("{}",e)}
        }
    }

    fn run(&self) {
        let mut stdin = BufferedReader::new(stdin());
        let mut history: String = String::from_str("");
        loop {
            old_io::stdio::print(self.cmd_prompt.as_slice());
            old_io::stdio::flush();

            let line = stdin.read_line().unwrap();
            let cmd_line = line.trim();
            let program = cmd_line.splitn(1, ' ').nth(0).expect("no program");
            let mut char_vec: Vec<char> = cmd_line.chars().collect();
            if char_vec.len()>0{
                match char_vec.last().unwrap(){
                    &'&' => {
                        let mut s:String=String::from_str(cmd_line);
                        s.remove(cmd_line.len()-1);
                        let subtask =thread::spawn(move ||{
                            Shell::new("").run_cmdline(s.as_slice(),String::new());
                        });
                        continue;
                    }
                    _  =>(),
                }}
            match program {
                ""          =>  { continue; }
                "cd"        =>  { self.cd( 
                        match cmd_line.splitn(1, ' ').nth(1){//check if the path is empty or not
                            Some(x) => x,
                            None    => "."});}
                "history"   =>  {self.run_cmdline(cmd_line,history.clone())}//store history in a string and pass it as a parameter.
                "exit"      =>  { return; }
                _           =>  { self.run_cmdline(cmd_line,String::new()); }
            }
            history.push_str(cmd_line);
            history.push('\n');

        }
    }

    //cmd_line is the raw input, extra_info is a string to carry any extra information from the higher level function
    fn run_cmdline(&self,cmd_line: &'a str,extra_info: String) {
        let  cmd_vec: Vec<&'a str> = cmd_line.split('|').filter_map(|x| {
            if x == "" {
                None
            } else {
                Some(x)
            }
        }).collect();
        let mut count=0;
        let (init_sender,init_receiver)=channel::<pack>();
        let mut last_receiver = init_receiver;
        
                

        for cmd in cmd_vec.iter()
        {
            if count== cmd_vec.len()-1{
                break;
            }
            let cmd_string= cmd.to_string();
            count+=1;
            let mut pipe_state= pipe_sign::pipe_both;
            if cmd_vec.len()==1{
                pipe_state=pipe_sign::pipe_none;
            }
            else if count==1{
                pipe_state=pipe_sign::pipe_out;
            }
            else if count==cmd_vec.len(){
                pipe_state=pipe_sign::pipe_in;
            }

            let (sender, receiver) = channel::<pack>();
            let info=extra_info.clone();

            thread::spawn(move|| {
                Shell::run_cmd(cmd_string,sender,last_receiver,pipe_state,info);
            });  
            last_receiver=receiver;
        }
        let (final_sender,final_receiver)=channel::<pack>();
        let last_pipe_state = match count
        {
            0 =>(pipe_sign::pipe_none),
            _ =>(pipe_sign::pipe_in),
        };
        Shell::run_cmd(cmd_vec[cmd_vec.len()-1].to_string(),final_sender,last_receiver,last_pipe_state,extra_info.clone());  
    }
    //cmd_line in this function is a single cmd inside the raw input which is splited by "|", for
    //example ls -l. here is sender and receiver between itself and next and former command,
    //extra_info keep carrying extra information
    fn run_cmd(cmd_line:String,sender : Sender<pack>,rec: Receiver<pack>,pipe_state: pipe_sign,extra_info:String) {
        // split the command on " " to get the program name and the arguments
        let argv_line: Vec<&str> = cmd_line.as_slice().split(' ').filter_map(|x| {
            if x == "" {
                None
            } else {
                Some(x)
            }
        }).collect();
        let program = match argv_line.first() {
            Some(&program) => (program),
            None => (""),};
        let argv = argv_line.tail();


        let mut redirect :redirection_sign =redirection_sign::no; 
        let mut target = Vec::new();
        let mut my_argv=Vec::new();
        for n in 0..argv.len()//split on first > or <
        {
            if argv[n]==">" {
                redirect=redirection_sign::output;
            }
            else if argv[n]=="<" {
                redirect=redirection_sign::input;

            }
            else {
                match redirect
                { 
                    redirection_sign::no => my_argv.push(argv[n]),               
                        _ => target.push(argv[n]),
                }
            }
        }


        if Shell::cmd_exists(program) {
            let mut cmd = match Command::new(program).args(&my_argv).stdin(Stdio::capture()).stdout(Stdio::capture()).stderr(Stdio::capture()).spawn() {
                Err(why) => panic!("couldn't spawn wc: {}", why),
                Ok(cmd) => cmd,
            };
            let mut stdout=cmd.stdout.unwrap();
            let mut stderr=cmd.stderr.unwrap();

            match redirect  // do input and output depending on the redirection status
            {
                redirection_sign::no => {//if no redirection
                    match pipe_state{
                        pipe_sign::pipe_in =>{// if it only have pipe in, means it is the last command in the chain
                            {
                                let mut stdin = cmd.stdin.unwrap();

                                loop{

                                    let input_buf = rec.recv().unwrap();
                                    match stdin.write(&input_buf.content[0..input_buf.size]) {
                                        Err(why) => (break),
                                        Ok(_) => (),
                                    };
                                    if input_buf.end_flag==true{
                                        break;
                                    }
                                } 
                            }
                            
                            let guard1=thread::scoped(move || {
                                loop{
                                    let mut buf: [u8;MSG_S]=[0;MSG_S];
                                    let value=match stdout.read(&mut buf){
                                        Err(why) => panic!("couldn't read  stdout: {}", why),
                                        Ok(value) => (value),                    };
                                    let str_buf=String::from_utf8_lossy(&buf[0..value]);
                                    print!("{}",str_buf);
                                    if value==0{
                                        break;
                                    }
                                }});
                            let guard2=thread::scoped(move || {
                                loop{
                                    let mut buf: [u8;MSG_S]=[0;MSG_S];
                                    let value=match stderr.read(&mut buf){
                                        Err(why) => panic!("couldn't read  stdout: {}", why),
                                        Ok(value) => (value),                    };
                                    let str_buf=String::from_utf8_lossy(&buf[0..value]);
                                    print!("{}",str_buf);
                                    if value==0{
                                        break;
                                    }

                                }});
                        }
                        pipe_sign::pipe_out =>{ //if it only have pipe out, means it is the last cmd in the chain
                            let mut buf =[0;MSG_S];
                            thread::spawn(move || {
                                loop{
                                    let value = match stdout.read(&mut buf){
                                        Err(why) => (0),
                                        Ok(size) => size};

                                    let info =pack{content: buf, end_flag: value<MSG_S, size: value };
                                                                       
                                    match sender.send(info){
                                        Err(why) => (break),
                                        Ok(_) =>(),
                                    };
                                    if value<MSG_S {
                                        break;}
                                }});
                        }
                        pipe_sign::pipe_both =>{// it is the middle ones
                            {
                                let mut stdin = cmd.stdin.unwrap();
                                loop{
                                    let input_buf = rec.recv().unwrap();
                                    match stdin.write(&input_buf.content[0..input_buf.size]) {
                                        Err(why) => (break),
                                        Ok(_) => (),
                                    };
                                    if input_buf.end_flag==true{
                                        break;
                                    }
                                } 
                            }

                            let mut buf =[0;MSG_S];
                            thread::spawn(move || {
                                loop{
                                    let value = match stdout.read(&mut buf){
                                        Err(why) => (0),
                                        Ok(size) => size};

                                    let info =pack{content: buf, end_flag: value<MSG_S,size: value };
                                    match sender.send(info){
                                        Err(why) => (break),
                                        Ok(_) =>(),
                                    };
                                    if value<MSG_S {
                                        break;}
                                }});
                        }
                        pipe_sign::pipe_none =>{// no pipe at all
                            thread::scoped(move || {
                                loop{
                                    let mut buf: [u8;MSG_S]=[0;MSG_S];
                                    let value=match stdout.read(&mut buf){
                                        Err(why) => panic!("couldn't read  stdout: {}", why),
                                        Ok(value) => (value),                    };
                                    let str_buf=String::from_utf8_lossy(&buf[0..value]);
                                    print!("{}",str_buf);
                                    if value==0{
                                        break;
                                    }
                                }});
                            thread::scoped(move || {
                                loop{
                                    let mut buf: [u8;MSG_S]=[0;MSG_S];
                                    let value=match stderr.read(&mut buf){
                                        Err(why) => panic!("couldn't read  stdout: {}", why),
                                        Ok(value) => (value),                    };
                                    let str_buf=String::from_utf8_lossy(&buf[0..value]);
                                    print!("{}",str_buf);
                                    if value==0{
                                        break;
                                    }

                                }});
                        }
                    }
                }

                redirection_sign::output =>{// if there are redirection and pipe at the same time, redirection will overwrite part of the input and output made by pipe
                    match pipe_state{
                        pipe_sign::pipe_in | pipe_sign::pipe_both =>{//overwrite the stdout, redirect output to the file 
                            {
                                let mut stdin = cmd.stdin.unwrap();

                                loop{

                                    let input_buf = rec.recv().unwrap();
                                    match stdin.write(&input_buf.content[0..input_buf.size]) {
                                        Err(why) => (break),
                                        Ok(_) => (),
                                    };
                                    if input_buf.end_flag==true{
                                        break;
                                    }
                                } 
                            }
                        }
                        _ =>(),

                    }


                    let mut f=File::create(&Path::new(target[0])).unwrap();
                    thread::spawn(move||{
                        let mut buf:[u8;MSG_S]=[0;MSG_S];
                        loop{
                            let value=match stdout.read(&mut buf){
                                Err(why) => panic!("couldn't read  stdout: {}", why),
                                Ok(value) => (value),                    };
                            f.write(&buf[0..value]);
                            if(value==0){
                                break;
                            }
                        }
                    });
                }

                redirection_sign::input  =>{// ignore the pipe in information, read from file
                    let mut f= match File::open(&Path::new(target[0]))              
                    {
                        Err(why) => {

                            print!("couldn't write to wc stdin: {}", why);
                            return;
                        }
                        Ok(f) => f,
                    };
                    let mut input_buf = String::new();

                    input_buf= match f.read_to_string(){
                        Err(why) => (String::new()),//panic!("couldn't write to input_buf: {}", why),
                        Ok(string) => (string),                    };
                    //input section above
                    match cmd.stdin.unwrap().write_all(input_buf.as_bytes()) {
                        Err(why) => panic!("couldn't write to wc stdin: {}", why),
                        Ok(_) => (),
                    } 
                    match pipe_state{
                        pipe_sign::pipe_out | pipe_sign::pipe_both => {
                            let mut buf =[0;MSG_S];
                            thread::spawn(move || {
                                loop{
                                    let value = match stdout.read(&mut buf){
                                        Err(why) => (0),
                                        Ok(size) => size};

                                    let info =pack{content: buf, end_flag: value<MSG_S, size: value };
                                    match sender.send(info){
                                        Err(why) => (break),
                                        Ok(_) =>(),
                                    };
                                    if value<MSG_S {
                                        break;}
                                }});
                        }
                        _ =>{
                            thread::scoped(move || {
                                loop{
                                    let mut buf: [u8;MSG_S]=[0;MSG_S];
                                    let value=match stdout.read(&mut buf){
                                        Err(why) => panic!("couldn't read  stdout: {}", why),
                                        Ok(value) => (value),                    };
                                    let str_buf=String::from_utf8_lossy(&buf[0..value]);
                                    print!("{}",str_buf);
                                    if value==0{
                                        break;
                                    }
                                }});
                            thread::scoped(move || {
                                loop{
                                    let mut buf: [u8;MSG_S]=[0;MSG_S];
                                    let value=match stderr.read(&mut buf){
                                        Err(why) => panic!("couldn't read  stdout: {}", why),
                                        Ok(value) => (value),                    };
                                    let str_buf=String::from_utf8_lossy(&buf[0..value]);
                                    print!("{}",str_buf);
                                    if value==0{
                                        break;
                                    }

                                }});
                        },
                    }
                }
            }                        
        } 
        else {
            match program {// deal with history cmd
                "history" =>{
                    match  pipe_state{
                        pipe_sign::pipe_out =>{                        
                            let mut buf =[0;MSG_S];
                            thread::spawn(move || {
                                let mut start=0;
                                let mut end = MSG_S;
                                loop{
                                    let mut i=0;
                                    let mut flag=false;
                                    if end> extra_info.clone().into_bytes().len(){
                                        end=extra_info.clone().into_bytes().len();
                                        flag=true;
                                    }
                                    for &x in extra_info.clone().into_bytes()[start..end].iter(){
                                        buf[i] = x;
                                        i+=1;
                                    }
                                    start+=MSG_S;
                                    end+=MSG_S;
                                    let info =pack{content: buf, end_flag: flag, size:end-start };
                                    match sender.send(info){
                                        Err(why) => (break),
                                        Ok(_) =>(),
                                    };   
                                    if flag{
                                        break;
                                    }
                                }});

                        }
                        _  =>{
                            println!("{}",extra_info);
                        }

                    }
                },
                _   =>  {println!("{}: command not found", program);}

            }
        }
    }

    fn cmd_exists(cmd_path: &str) -> bool {
        Command::new("which").arg(cmd_path).stdout(Stdio::capture()).status().unwrap().success()
    }
}

fn get_cmdline_from_args() -> Option<String> {
    /* Begin processing program arguments and initiate the parameters. */
    let args = os::args();

    let opts = &[
        getopts::optopt("c", "", "-c ls ", "-c follows the command ")
        ];

    getopts::getopts(args.tail(), opts).unwrap().opt_str("c")
}

fn main() {
    let opt_cmd_line = get_cmdline_from_args();

    match opt_cmd_line {
        Some(cmd_line) => Shell::new("").run_cmdline(cmd_line.as_slice(),String::new()),// if the mian receive argument from concole, execute it directly without entering gash, and run that command only
        None           => Shell::new("gash > ").run(),//other wise start gash. 
    }
}
