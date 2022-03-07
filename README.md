# Home Assistant desktop shortcuts
Create keyboard shortcuts for Home Assistant services.
## Download the app
Go to the releases page and download the latest version or [click here to download](https://github.com/Winor/homeassistant_hotkeys/releases/latest/download/homeassistant_hotkeys.exe)
## Running the app:
### You need:
- Windows 7+ (Only tested on Windows 11)
- Long-Lived Access Token
- Running Home assistant server
### To create a Long-Lived Access Token:
- Login to Home Assistant web interface
- Go to Profile --> Long-Lived Access Tokens and create a token
- Copy the token and paste it in your config.yaml file
## Running the app for the first time
When running the App for the first time, it'll generate a config file at
`%appdata%/hass_hotkeys/config/config.yaml` and quit. Edit the file to match your setup before starting the app again.


## Example config.yaml configuration:
Toggle entity ``light.lab_lights`` when pressing ``LeftControl`` & ``R``
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
You may add as many actions as you wish! Make sure to restart the app every time you edit the config file for the changes to take effect.

**NOTE:** This project is a work in progress.
## TODO:
- [ ] linux support
- [ ] UI for configuration 