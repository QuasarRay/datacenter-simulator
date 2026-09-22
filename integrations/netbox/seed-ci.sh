#!/usr/bin/env bash
# Seed only the disposable localhost NetBox created by compose.ci.yml.
set -euo pipefail
api=http://127.0.0.1:8000/api
NETBOX_TOKEN=nbt_SimTestKey01.0123456789012345678901234567890123456789
post() {
  curl --fail-with-body --silent --show-error --max-time 30 \
    -H "Authorization: Bearer $NETBOX_TOKEN" -H 'Content-Type: application/json' \
    "$api/$1/" --data "$2"
}
site=$(post dcim/sites '{"name":"Simulator","slug":"simulator","status":"active"}' | jq -er .id)
manufacturer=$(post dcim/manufacturers '{"name":"Simulator","slug":"simulator"}' | jq -er .id)
type=$(post dcim/device-types "$(jq -nc --argjson m "$manufacturer" '{manufacturer:$m,model:"CPU lab",slug:"cpu-lab"}')" | jq -er .id)
host_role=$(post dcim/device-roles '{"name":"GPU server","slug":"gpu-server","color":"2196f3"}' | jq -er .id)
switch_role=$(post dcim/device-roles '{"name":"IB switch","slug":"ib-switch","color":"4caf50"}' | jq -er .id)
for field in simulator_gpu_profile simulator_kubernetes_node; do
  post extras/custom-fields "$(jq -nc --arg n "$field" '{name:$n,type:"text",object_types:["dcim.device"]}')" > /dev/null
done
post extras/custom-fields '{"name":"simulator_gpu_count","type":"integer","object_types":["dcim.device"]}' > /dev/null
ids=()
for n in 1 2 3; do
  role=$host_role
  custom='{}'
  if [ "$n" = 3 ]; then role=$switch_role; else
    worker=simulator-worker
    if [ "$n" = 2 ]; then worker=simulator-worker2; fi
    custom=$(jq -nc --arg worker "$worker" '{simulator_gpu_profile:"t4",simulator_gpu_count:2,simulator_kubernetes_node:$worker}')
  fi
  ids+=("$(post dcim/devices "$(jq -nc --arg name "node-$n.lab.example" --argjson site "$site" --argjson type "$type" --argjson role "$role" --argjson custom "$custom" '{name:$name,site:$site,device_type:$type,role:$role,status:"active",custom_fields:$custom}')" | jq -er .id)")
done
ports=()
for n in 0 1 2 2; do
  ports+=("$(post dcim/interfaces "$(jq -nc --argjson device "${ids[$n]}" --arg name "IB1/${#ports[@]}" '{device:$device,name:$name,type:"infiniband-hdr",enabled:true,mgmt_only:false,speed:200000000}')" | jq -er .id)")
done
for n in 0 1; do
  post dcim/cables "$(jq -nc --argjson a "${ports[$n]}" --argjson b "${ports[$((n+2))]}" '{status:"connected",a_terminations:[{object_type:"dcim.interface",object_id:$a}],b_terminations:[{object_type:"dcim.interface",object_id:$b}]}')" > /dev/null
done
# Give the Rust importer a read-only v2 token. No Python imports/FFI are involved.
readonly_token=$(post users/tokens '{"user":1,"version":2,"write_enabled":false,"description":"Simulator read-only CI"}')
jq -er '"nbt_" + .key + "." + .token' <<< "$readonly_token"
