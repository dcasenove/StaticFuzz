#/usr/bin/python3

#Prune targets based on FuzzBench reports in .html format
#Inputs: txt file with filename:line format targets
#        dir with .html FuzzBench reports
#Ex run python3 prune_targets.py benchmark.custom_targets.txt ./
lcms_infer.custom_targets.txt ./

from html_table_parser import HTMLTableParser
from pprint import pprint
import sys

report = sys.argv[1]
fpath = sys.argv[2]

def parse(fpath,to_check):

    to_check = to_check.split('/')[-1]
    filename = to_check.split(':')[0]
    line = int(to_check.split(':')[-1].rstrip("\n"))
    
    print(line)
    print(filename)
    try:
        file = open(fpath + filename + ".html", "r", encoding ='ISO-8859-1');
        xhtml = file.read()
    except FileNotFoundError:
        return

    p = HTMLTableParser()
    p.feed(xhtml)

    if p.tables[0][line][1] != '0' and  p.tables[0][line][1] != '':
        print(to_check.rstrip("\n"))
        print(p.tables[0][line])
        file_out.write(to_check)

file_report = open(report, "r");
file_out = open(report + ".parsed", "a")
for line in file_report:
    parse(fpath, line)

