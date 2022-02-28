# Home Assistant desktop shortcuts
Create keyboard shortcuts for Home Assistant services.

## example config.yaml configuration:
Togle entity ``light.lab_lights`` when pressing ``LeftControl`` & ``R``
```yaml
hass_host: '' #replace your home assistant ip or domain
hass_port: 8123 #replace with your home assistant websocket port
hass_token: '' #replace with a long lived access token
actions:
  - action_type: call_service
    description: Toggle Lab lights when pressing LeftCtrl & R
    keys:
      - LeftControl
      - R
    domain: light
    service: toggle
    service_data:
       entity_id: light.lab_lights
```
You may add as many actions as you wish!

**NOTE:** This project is work in progress.