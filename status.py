#!/usr/bin/env python3
import os
import matplotlib
matplotlib.use('agg')
import matplotlib.pylab as plt
import numpy as np
import yaml
from datetime import datetime
data=np.fromfile('/dev/shm/dump.bin', dtype=np.complex64).reshape([-1,8192])

stations=yaml.load(open('/dev/shm/cfg.yaml'), Loader=yaml.FullLoader)['stations']

img_dir='/dev/shm/imgs'
if not os.path.exists(img_dir):
    os.mkdir(img_dir)
baseline={}
freqs=np.arange(8192)/8192*200
chskip=128
for l in open('/dev/shm/corr_prod.txt'):
    n, a1,a2=[int(i) for i in l.split()]
    baseline[n]=(a1,a2)
    if a1==a2:
        print(stations[a1])
        plt.figure()
        plt.plot(freqs[chskip:], 10*np.log10(np.abs(data[n, chskip:])))
        #plt.show()    
        plt.title(stations[a1])
        plt.tight_layout()
        plt.savefig("{0}/{1}.png".format(img_dir, stations[a1]))
        plt.close()
