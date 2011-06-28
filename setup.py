#!/usr/bin/env python

from distutils.core import setup
import os
import sys

print sys.argv[1]

#if not os.path.exists('monitor/bin/hydra-monitor') and \
#       os.path.exists('monitor/bin/hydra-monitor.py'):
#    os.link('monitor/bin/hydra-monitor.py', 'monitor/bin/hydra-monitor')

setup(name = 'hydra-server',
      license = 'Proprietary',
      version = '0.2',
      author = "Whamcloud, Inc.",
      author_email = "info@whamcloud.com",
      description = 'The Whamcloud Lustre Monitoring and Adminisration Interface',
      long_description = 'This is the Whamcloud Monitoring and Adminstration Interface',
      url = 'http://www.whamcloud.com/',
      packages = ['', 'monitor', 'monitor/lib'],
      scripts = ['monitor/bin/hydra-monitor.py'],
      package_data={'monitor': ['templates/*', 'static/css/*.css',
                                'static/css/ui-lightness/*.css',
                                'static/css/ui-lightness/images/*',
                                'static/images/*', 'static/js/*'], },
      )

#if os.path.exists('monitor/bin/hydra-monitor'):
#    os.unlink('monitor/bin/hydra-monitor')
