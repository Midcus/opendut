# Setup

- Download the opendut-edgar binary for your target from the openDuT GitHub project: https://github.com/eclipse-opendut/opendut/releases
- Unpack the archive on your target system.
- If you want to use CAN, follow the steps in [CAN Setup](#can-setup) before continuing.
- If you want to use a self-hosted backend server without DNS name or with a self-signed certificate, follow the steps in [Self-Hosted Backend Server](#self-hosted-backend-server) before continuing.


- EDGAR comes with a scripted setup, which you can initiate by running:  
  ```shell
  opendut-edgar setup managed <SETUP-STRING>
  ```  
You can get the `<SETUP-STRING>` from LEA or CLEO after creating a Peer.

This will configure your operating system and start the *EDGAR Service*, which will receive its configuration from *CARL*.

## Download EDGAR from CARL
You can download the EDGAR binary as tarball from one of CARLs endpoints.

The archive can be requested at `https://{CARL-HOST}/api/edgar/{architecture}/download`.

Available architectures are:
- x86_64-unknown-linux-gnu
- armv7-unknown-linux-gnueabihf
- aarch64-unknown-linux-gnu

Once downloaded, extract the files with the command `tar xvf opendut-edgar-{architecture}.tar.gz`. It will then be extracted into
the folder which is the current work directory. You might want to use another directory of your choice.


## CAN Setup
If you want to use CAN, it is mandatory to set the environment variable `OPENDUT_EDGAR_SERVICE_USER` as follows:
```shell
export OPENDUT_EDGAR_SERVICE_USER=root
```

When a cluster is deployed, EDGAR automatically creates a virtual CAN interface (by default: `br-vcan-opendut`) that is used as a bridge between Cannelloni instances and physical CAN interfaces. EDGAR automatically connects all CAN interfaces defined for the peer in CARL to this bridge interface. 

This also works with virtual CAN interfaces, so if you do not have a physical CAN interface and want to test the CAN functionality nevertheless, you can create a virtual CAN interface as follows. Afterwards, you will need to configure it for the peer in CARL.

```shell 
# Optionally, replace vcan0 with another name
ip link add dev vcan0 type vcan
ip link set dev vcan0 up
  ```

### Preparation
EDGAR relies on the Linux socketcan stack to perform local CAN routing and uses Cannelloni for CAN routing between EDGARs.
Therefore, we have some dependencies.
1. Install the following packages:
  ```shell
  sudo apt install -y can-utils
  ```
2. Download Cannelloni from here: https://github.com/eclipse-opendut/cannelloni/releases/
3. Unpack the Cannelloni tarball and copy the files into your filesystem like so:
  ```shell
  sudo cp libcannelloni-common.so.0 /lib/
  sudo cp libsctp.so* /lib/
  sudo cp cannelloni /usr/local/bin/
  ```

### Testing
When you configured everything and deployed the cluster, you can test the CAN connection between different EDGARs as follows:
- Execute on EDGAR leader, assuming the configured CAN interface on it is `can0`:
  ```shell
  candump -d can0
  ```
- On EDGAR peer execute (again, assuming can0 is configured here):
  ```shell
  cansend can0 01a#01020304
  ```
  Now you should see a can frame on leader side:
  ```text
  root@host:~# candump -d can0
  can0  01A   [4]  01 02 03 04
  ```

## Self-Hosted Backend Server

### DNS
If your backend server does not have a public DNS entry, you will need to adjust the `/etc/hosts/` file, by appending entries like this (using your server's IP address):
```
123.456.789.101 opendut.local
123.456.789.101 carl.opendut.local
123.456.789.101 auth.opendut.local
123.456.789.101 netbird.opendut.local
123.456.789.101 netbird-api.opendut.local
123.456.789.101 signal.opendut.local
```

Now the following command should complete without errors:
```
ping carl.opendut.local
```

### Self-Signed Certificate with Unmanaged Setup
If you plan to use the unmanaged setup and your NetBird server uses a self-signed certificate, follow these steps:

1. Create the certificate directory on the OLU: `mkdir -p /usr/local/share/ca-certificates/`

2. Copy your NetBird server certificate onto the OLU, for example, by running the following from outside the OLU:  
   ```sh
   scp certificate.crt root@10.10.4.1:/usr/local/share/ca-certificates/
   ```  
   Ensure that the certificate has a file extension of "crt".

3. Run `update-ca-certificates` on the OLU.  
   It should output "1 added", if everything works correctly.  

4. Now the following commands should complete without errors:
   ```
   curl https://netbird-api.opendut.local
   ```

## Troubleshooting
- In case of issues during the managed setup see:
  ```shell
  journalctl -u opendut-edgar.service
  ```
- Sometimes it might be necessary to stop and re-start the EDGAR service:
  ```shell
  # Stop service
  sudo systemctl stop opendut-edgar.service
  # Start service
  sudo systemctl start opendut-edgar.service
  # Restart service
  sudo systemctl restart opendut-edgar.service
  # Check status
  systemctl status opendut-edgar.service
  ```

- Netbird service on host machine. It might happen that Netbird is not able to connect, in that case stop it and re-run EDGAR managed setup:
  ```shell
  # Stop service
  sudo systemctl stop netbird.service
  # Start service
  sudo systemctl start netbird.service
  ```

- More log files / statements can be found in the corresponding Docker containers:
  ```shell
  docker logs localenv-keycloak-1
  docker logs localenv-carl-1
  # To display all running containers use:
  docker ps
  ```

- Netbird logs are available on host machine 
  ```shell
  cat /var/lib/netbird/client.log
  cat /var/lib/netbird/netbird.err
  cat /var/lib/netbird/netbird.out
  ```

- EDGAR might start with an old IP, different from command `sudo wg` would print. In that particular case
stop netbird service and opendut-edgar service and re-run the setup. This might happen to all
EDGARs. If this is not enough, and it keeps getting the old IP, it is necessary to set up all
devices and clusters from scratch.
  ```shell
  sudo wg
  ```

- If this ERROR appears: ERROR opendut_edgar::service::cannelloni_manager: Failure while invoking command line program 'cannelloni': 'No such file or directory (os error 2)'.
(OPTIONAL) Copy cannelloni to custom location. This is the way to go, if the step before is
    not possible to be done. This can happen for whatever reason, i.e. missing root rights.
  ```shell
  export LD_LIBRARY_PATH=/your-path-to-cannelloni/cannelloni (just folder, not the binary)
  export PATH={all_other_PATH_variables}:/your-path-to-cannelloni/cannelloni
  vim /etc/systemd/system/opendut-edgar.service 
  systemctl daemon-reload
  systemctl restart opendut-edgar.service
  ```

  Copy the two Environment variables into your opendut-edgar.service file.
  ```text
  [Unit]
  ...
  
  [Service]
  ...
  Environment="LD_LIBRARY_PATH=/opt/opendut/edgar/cannelloni"
  Environment="PATH=/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin:/opt/opendut/edgar/cannelloni"
  
  
  [Install]
  ...
  ```
