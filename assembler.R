#!/usr/bin/env Rscript
# Script to parse Hack assembly (.asm files) and maybe in the end real assembler.
library(bitops)
library(hash)

checkReadable <- function(filename) {
    res <- file.access(names=filename, mode=4) == 0
    if (!res) {
        warning(paste(filename, "is not readable", sep=" "))
    }
    res
}

arguments <- matrix(c(
    'help', 'h', 0, "logical",
    'debug', 'd', 1, "character",
    'asm'  , 'a', 1, "character"
), ncol=4, byrow=T)

library(getopt)
opt <- getopt(arguments)

if (!is.null(opt$help)) {
    stop(getopt(arguments, command=get_Rscript_filename(), usage=TRUE))
}
if (is.null(opt$asm)) {
    stop("Missing asm [-a filename]\n")
}
if (!is.null(opt$debug)) {
    save.image(file=opt$debug)
}

#data@decoder:~/Dropbox/hacknight/blu$ sed -e 's/\s//g' < jmp  | tr ":" "\t" | grep -v '{' | grep -v '}' | cut -f 1 | tr "\n" ","
jump_v_k <- c("FUCKHASHES","JGT","JEQ","JGE","JLT","JNE","JLE","JMP")
jump_v_v <- c("000","001","010","011","100","101","110","111")
comp_v_k <- c("0","1","-1","D","A","!D","!A","-D","-A","D+1","A+1","D-1","A-1","D+A","D-A","A-D","D&A","D|A","M","!M","-M","M+1","M-1","D+M","D-M","M-D","D&M","D|M")
comp_v_v <- c("0101010","0111111","0111010","0001100","0110000","0001101","0110001","0001111","0110011","0011111","0110111","0001110","0110010","0000010","0010011","0000111","0000000","0010101","1110000","1110001","1110011","1110111","1110010","1000010","1010011","1000111","1000000","1010101")
jump <- hash(keys=jump_v_k, values=jump_v_v)
computation <- hash(keys=comp_v_k, values=comp_v_v)

encode_computation <- function(k) {
    computation[[k]]
}

encode_jump <- function(k) {
    if (k == "") {
        k <- "FUCKHASHES"
    }
    jump[[k]]
}

encode_destination <- function(s) {
    result <- 0
    adm <- function(c) {
        if (c == "A") {
            result <<- bitOr(result, 4)
        } else if (c == "D") {
            result <<- bitOr(result, 2)
        } else if (c == "M") {
            result <<- bitOr(result, 1)
        }
    }
    lapply(unlist(strsplit(s, split=NULL)), adm)
    binary_result <- sapply(strsplit(paste(rev(intToBits(result))),"", fixed=TRUE),`[[`,2)
    # FUUUCK R. http://stackoverflow.com/questions/6614283/converting-decimal-to-binary-in-r
    paste(tail(binary_result, n=3), collapse="")
}

encode_computation_jump <- function(s) {
    parts = unlist(strsplit(s, ";", fixed=TRUE))
    if (length(parts) == 1) {
        c(encode_computation(parts[1]), "000")
    } else {
        c(encode_computation(parts[1]), encode_jump(parts[2]))
    }
}

decode_instruction <- function(instr) {
    single_chars <- unlist(strsplit(instr, split=NULL))
    if (single_chars[1] == '@') {
        # Istruzione A.
        #number = int(line[1:])
        number <- paste0(tail(single_chars, n=-1), collapse="")
        binary_result <- sapply(strsplit(paste(rev(intToBits(number))),"", fixed=TRUE),`[[`,2)
        paste0("0", paste0(tail(binary_result, n=15), collapse=""))
        #print "0" + binary[-15:] + "  #" + line
    } else {
        # Istruzione D.
        parts <- unlist(strsplit(instr, split="=", fixed=TRUE))
        if (length(parts) == 1) {
            d <- "000"
            cj <- encode_computation_jump(parts[1])
        } else {
            d <- encode_destination(parts[1])
            cj <- encode_computation_jump(parts[2])
        }
        paste0("111", cj[1], d, cj[2])
    }
}

lines <- readLines(opt$asm)
uncomm_trimmed_lines <- lapply(lines, function(x) { gsub("^\\s+|\\s+$", "", unlist(strsplit(x, split="//", fixed=TRUE))[1]) })
polished_asm <- uncomm_trimmed_lines[sapply(uncomm_trimmed_lines,function(x) {x!=""})] 
hack <- lapply(polished_asm, decode_instruction)
cat(paste0(paste0(paste0(hack, "  #"), polished_asm), collapse="\n"))
cat("\n")
