import cv2 as cv
import numpy as np
from matplotlib import pyplot as plt
import seaborn as sns

def sorting (x):
    return x[0]
mask = None
img = cv.imread('/home/tima/rubbish/bottle/bottle0.jpg')
hist = cv.calcHist([img],[0], mask , [256], [0, 256])
#print(hist)

histr1 = []

for i in range(len(hist)):
    histr1.append([hist[i][0],i])

histr1 = sorted(histr1, key= sorting)


histx = []
histy = []
zeros = np.arange(256)
for i in range (len(histr1)):
    histy.append(histr1[i][0])
    histx.append(str(histr1[i][1]))
print(histx)

b = sns.lineplot(data=histy,color = 'b')

b.set_xticklabels(histx)

hist = cv.calcHist([img],[1], mask , [256], [0, 256])
#print(hist)

histr1 = []

for i in range(len(hist)):
    histr1.append([hist[i][0],i])

histr1 = sorted(histr1, key= sorting)


histx = []
histy = []
zeros = np.arange(256)
for i in range (len(histr1)):
    histy.append(histr1[i][0])
    histx.append(str(histr1[i][1]))
print(histx)

g = sns.lineplot(data=histy,color = 'g')

g.set_xticklabels(histx)

hist = cv.calcHist([img],[2], mask , [256], [0, 256])
#print(hist)

histr1 = []

for i in range(len(hist)):
    histr1.append([hist[i][0],i])

histr1 = sorted(histr1, key= sorting)


histx = []
histy = []
zeros = np.arange(256)
for i in range (len(histr1)):
    histy.append(histr1[i][0])
    histx.append(str(histr1[i][1]))
print(histx)

r = sns.lineplot(data=histy,color = 'r')

r.set_xticklabels(histx)
#g1 = sns.lineplot(data = hist)
#g1.set_xticklabels(np.arange(256))
#plt.bar(histx,histy)

plt.show()
