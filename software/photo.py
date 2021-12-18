import cv2 as cv
import os

def sorting (file):
    i = 0
    while ord(file[i]) >  57:
        i = i + 1
    chislo = 0
    while ord(file[i]) >= 48 and ord(file[i]) <= 57:
        chislo *= 10
        chislo += int(file[i])
        i += 1
    return chislo

def get_last_number (name_of_rubbish):
    directory = '/home/tima/rubbish/{}'.format(name_of_rubbish)
    files = os.listdir(directory) 
    files.sort(key = sorting)
    #print(files)
    return sorting(files[-1]) # получение значения последнего файла

paper_number = get_last_number('paper')
tin_number = get_last_number('tin')
bottle_number = get_last_number('bottle')

'''file = open('text.txt','r+')

zn = file.read()
zn = zn.split()
print(zn)

paper_number = int(zn[0])
tin_number = int(zn[1])
bottle_number = int(zn[2])'''

print(paper_number, tin_number, bottle_number)


cap = cv.VideoCapture(0)

try:
    while True:
        img = cv.resize(cap.read()[1],(640,480))

        #img_to_show = cv.resize(img, (640, 480))
        cv.imshow('video feed', img)
        
        i = cv.waitKey(1)

        if i == ord('p'):
            filename = 'paper/paper{}.jpg'.format(paper_number)
            cv.imwrite(filename, img)
            print(f"Saved {filename}")
            paper_number += 1
        elif i == ord('t'):
            filename = 'tin/tinb{}.jpg'.format(tin_number)
            cv.imwrite(filename, img)
            print(f"Saved {filename}")
            tin_number += 1
        elif i == ord('b'):
            filename = 'bottle/bottle{}.jpg'.format(bottle_number)
            cv.imwrite(filename, img)
            print(f"Saved {filename}")
            bottle_number += 1
        elif i == 27:
            break
except:
    import traceback
    traceback.print_exc()


