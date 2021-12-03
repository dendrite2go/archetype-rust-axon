#!/bin/echo Source this script:

export PROJECT_BIN="$(cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd)"
export PROJECT_DIR="$(dirname "${PROJECT_BIN}")"
export PROJECT_NAME="$(basename "${PROJECT_DIR}")"
export WORK_AREA="$(dirname "${PROJECT_DIR}")"

function remove-dirs() {
  local LIST="$1"
  shift
  echo "${LIST}" | tr ':' '\012' \
    | while read D
      do
        I="true"
        for R in "$@"
        do
          if [[ ".${D}" = ".${R}" ]]
          then
            I="false"
          fi
          if "${I}"
          then
            echo "${D}"
          fi
        done
      done \
    | tr '\012' : \
    | sed -e 's/:$//'
}

function find-bin-dirs() {
  local TOP="$1"
  local DEPTH="$2"
  find "${TOP}" -maxdepth "${DEPTH}" \( -type d -name node_modules -prune -type f \) -o -type d -name bin \
    | sed -e 's:/bin$:/!:' \
    | sort \
    | sed \
    | sed -e 's@/!$@/bin@'
}

M='3'

while read B
do
  PATH="$(remove-dirs "${PATH}" "${B}")"
  ## echo "Removed [${B}] from [${PATH}]"
done < <(find-bin-dirs "${WORK_AREA}" "$(($M + 1))")

PATH="$(find-bin-dirs "${PROJECT_DIR}" "${M}" | tr '\012' ':' | sed -e 's/:$//'):${PATH}"

if [ -f "${PROJECT_BIN}/bashrc.sh" ]
then
  source "${PROJECT_BIN}/bashrc.sh"
fi

PS1="${PROJECT_NAME}:\W \u\$ "
echo -n -e "\033]0;${PROJECT_NAME}\a"