import { LoadedMap } from "..";
import SimpleRadarDefault from "./map_style_simple_radar.png";
import OfficialDefault from "./map_style_cs2.png";

export default {
    mapName: "de_dust",
    displayName: "Dust",

    pos_x: -2476, // upper left world coordinate
    pos_y: 3239,
    scale: 4.4,

    verticalSections: [
        {
            name: "default",
            altitudeMax: 10000,
            altitudeMin: -10000,
        }
    ],

    mapStyles: [
        {
            name: "SimpleRadar",
            map: {
                default: SimpleRadarDefault,
            }
        },
        {
            name: "Official",
            map: {
                default: OfficialDefault,
            }
        }
    ]
} satisfies LoadedMap;
