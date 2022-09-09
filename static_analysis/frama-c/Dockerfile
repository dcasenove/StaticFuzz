FROM ocaml/opam2:ubuntu-20.04

RUN sudo apt-get update
RUN opam depext -y frama-c
RUN opam install -y frama-c

CMD ["tail", "-f", "/dev/null"]
