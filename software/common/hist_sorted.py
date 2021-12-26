import cv2 as cv
import numpy as np
from matplotlib import pyplot as plt
#import seaborn as sns


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


def image (name):
    return cv.imread('/home/tima/rubbish/{}/{}.jpg'.format(typing(name),name))


def subplot (j, k, col, histx, histy, name, r):
    ax[j][k].plot(histx,histy,color = col)
    #ax[j][k] = sns.lineplot(data=histy,color = col)
    #ax[j, k].set_title('{} {}'.format(name, col))
    ax[j][k].set_xticklabels(histx)
    plt.setp(ax[j][k].get_xticklabels(), rotation = r, horizontalalignment='right', fontsize='x-small')


mask = None

name0 = 'paper0'
name1 = 'tin0'
name2 = 'bottle0'

fig , ax = plt.subplots(nrows = 3, ncols = 3, figsize = (25,25))
fig.suptitle('rubbish')

bgr = ('b', 'g', 'r')
names = (name0,name1,name2)
images = list(map(image,names))

for k, img in enumerate(images):
    for j, col in enumerate (bgr):
        hist = cv.calcHist([img], [j], mask , [256], [0, 256])
        histr1 = []

        for i in range(len(hist)):
            histr1.append([hist[i][0], i])

        histr1 = sorted(histr1, key = lambda x: x[0], reverse = True)

        histx = []
        histy = []

        for i in range (len(histr1)):
            histy.append(histr1[i][0])
            histx.append(str(histr1[i][1]))

        print(histx)
        
        subplot(j,k,col,histx[:40],histy[:40],name = names[k], r = 50)
        
plt.show()
