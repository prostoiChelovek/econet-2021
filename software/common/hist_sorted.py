import cv2 as cv
import numpy as np
from matplotlib import pyplot as plt
import seaborn as sns

def sorting (x):
    return x[0]
def typing (name):
    type = ''
    i = 0
    j = -1
    for i in range (len(name)):
        try:
            j = int(name[i])

        except :
            j = -1
            type += name[i]
            i += 1
            
        if j > -1:
            return type
mask = None
name = 'paper2'

img = cv.imread('/home/tima/rubbish/{}/{}.jpg'.format(typing(name),name))

#print(hist)
bgr = ('b','g','r')
for i, col in enumerate (bgr):
    hist = cv.calcHist([img],[i], mask , [256], [0, 256])
    histr1 = []

    for i in range(len(hist)):
        histr1.append([hist[i][0],i])

    histr1 = sorted(histr1, key= sorting)


    histx = []
    histy = []
    for i in range (len(histr1)):
        histy.append(histr1[i][0])
        histx.append(str(histr1[i][1]))
    print(histx)

    write_hist = sns.lineplot(data=histy,color = col)

    write_hist.set_xticklabels(histx)

    
    #print(hist)


plt.show()
