ARG FRR_BASE
FROM ${FRR_BASE}
USER root
RUN apk add --no-cache openssh iputils iproute2 \
    && adduser -D -s /usr/bin/vtysh ansible \
    && passwd -d ansible \
    && addgroup ansible frrvty \
    && install -d -m 0700 -o ansible -g ansible /home/ansible/.ssh \
    && sed -i 's/^bgpd=no/bgpd=yes/' /etc/frr/daemons \
    && printf '%s\n' 'service integrated-vtysh-config' > /etc/frr/vtysh.conf
COPY sshd_config /etc/ssh/sshd_config
COPY ex457-start /usr/local/sbin/ex457-start
COPY ex457-load /usr/local/sbin/ex457-load
RUN chmod 0755 /usr/local/sbin/ex457-start /usr/local/sbin/ex457-load
CMD ["/usr/local/sbin/ex457-start"]
