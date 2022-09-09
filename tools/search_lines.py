#!/usr/bin/python3

import os
import sys
import shutil
import subprocess
import csv

output_path = sys.argv[1]
cov_command = sys.argv[2] + " AFL_FILE"
cov_src = sys.argv[3]
queue_path = output_path + '/queue/'
targets_path = output_path + '/targets.txt'
tmp = output_path + '/tmp/'
tmp_cov = output_path + '/tmp/cov/'
tmp_path = output_path + '/tmp/queue/'

def checkLine(pos, pos_id, input_files, src_file, line):
    os.mkdir(tmp)
    os.mkdir(tmp_path)
    
    for i in range(pos):
        shutil.copy2(queue_path + input_files[i], tmp_path + input_files[i])
    
    #Get coverage on the whole tmp-dir
    process = subprocess.run(['./afl-cov', '--coverage-include-lines', '--coverage-at-exit', '--disable-lcov-web', '-e', cov_command , '-c', cov_src, '-d', tmp], stdout=subprocess.DEVNULL)
    
    #Query line
    process = subprocess.run(['./afl-cov', '--src-file', src_file, '--line-search', line, '-d', tmp], capture_output=False, stdout=subprocess.DEVNULL)
    
    #Clean up
    shutil.rmtree(tmp)
    if process.returncode == 0:
        return True
    else:
        return False


def checkAll(pos, input_files, target_file):
    os.mkdir(tmp)
    os.mkdir(tmp_path)
    found = []
    not_found = []

    for i in range(pos):
        shutil.copy2(queue_path + input_files[i], tmp_path + input_files[i])
    
    #Get coverage on the whole tmp-dir
    process = subprocess.run(['./afl-cov', '--coverage-include-lines', '--coverage-at-exit', '--disable-lcov-web', '-e', cov_command , '-c', cov_src, '-d', tmp], stdout=subprocess.DEVNULL)
    
    for targets in target_file:
    
        src_file = cov_src + targets.split(':')[0]
        line = targets.split(':')[-1].rstrip("\n")
        #Query line
        process = subprocess.run(['./afl-cov', '--src-file', src_file, '--line-search', line, '-d', tmp], capture_output=False, stdout=subprocess.DEVNULL)
        
        if process.returncode == 0:
            found.append(targets)
        else:
            not_found.append(targets)
    
    #Clean up
    shutil.rmtree(tmp)

    return found, not_found

def binarySearch(input_files, src_file, line):
    left = 0
    right = len(input_files)
    
    while left < right:
        mid = int(left + ((right - left) / 2 ))
        if(checkLine(mid, input_files[mid], input_files, src_file, line)):
            right = mid
        else:
            left = mid + 1
    return left

def searchTime(pos_id):
    csv_file = csv.reader(open(output_path + '/timestamps_delta.csv', "r"), delimiter=",")
    
    for row in csv_file:
        if row[0] == pos_id:
            return row[2]

#Get files
files = []

for filename in os.scandir(queue_path):
    if filename.is_file():
        split = filename.path.split("/")[-1]
        files.append(split)

#Sort ids
files.sort()

#Times and header
header = ['line', 'id' ,'time']
times = []
times.append(header)

target_file = open(targets_path, "r")

found, not_found = checkAll(len(files), files, target_file)

for targets in found:
    
    filename = cov_src + targets.split(':')[0]
    line = targets.split(':')[-1].rstrip("\n")

    print("[+] Line ", line, " in file: ", filename, " is reached")
    res = binarySearch(files, filename, line)
    time = searchTime(files[res-1])
    print("\t [+] Line ", line, " in file: ", filename, " executeb by ", files[res-1], " with time delta ", time)
    row = [filename + ":" + line, files[res-1], time]
    times.append(row)

for targets in not_found:
    
    filename = cov_src + targets.split(':')[0]
    line = targets.split(':')[-1].rstrip("\n")
    
    print("[-] Line ", line, " in file: ", filename, " is not reached")
    row = [filename + ":" + line, "-", "-"]

#Dump results to csv
target_times_path = output_path + '/target_times.csv'
target_times = open(target_times_path, 'w')
writer = csv.writer(target_times)
writer.writerows(times)
target_times.close()
