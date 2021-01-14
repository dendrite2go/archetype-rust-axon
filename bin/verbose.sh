#!/usr/bin/false

FLAGS_INHERIT=()

SILENT='true'
TRACE='false'

function message() {
	local TYPE="$1"
	shift
	echo "[${TYPE}] \$ ${SCRIPT}:" "$@" >&2
}

function error() {
	message 'ERROR' "$@"
	exit 1
}

function info() {
	message 'INFO' "$@"
}

function log() {
	"${SILENT}" || message 'DEBUG' "$@"
}

function trace() {
	"${TRACE}" && message 'TRACE' "$@" || true
}

if [ ".$1" = '.-v' ]
then
        SILENT='false'
        FLAGS_INHERIT[${#FLAGS_INHERIT[@]}]='-v'
        shift
        if [ ".$1" = '.-v' ]
        then
                TRACE='true'
                FLAGS_INHERIT[${#FLAGS_INHERIT[@]}]='-v'
                set -x
                shift
        fi
fi

if [ ".$1" = '.--' ]
then
    shift
fi

trace FLAGS_INHERIT: "${FLAGS_INHERIT[@]}"
