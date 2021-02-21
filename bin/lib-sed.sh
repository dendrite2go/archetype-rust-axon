#!/usr/bin/false

LIB_SED_STATUS='loading'

SED_EXT=-r
case $(uname) in
Darwin*)
        SED_EXT=-E
esac
export SED_EXT

LIB_SED_STATUS='loaded'
