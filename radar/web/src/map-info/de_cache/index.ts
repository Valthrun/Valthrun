import { LoadedMap } from "..";
import SimpleRadarDefault from "./map_style_simple_radar.png";
import OfficialDefault from "./map_style_cs2.png";

export default {
    mapName: "de_cache",
    displayName: "Cache",

    pos_x: -2020, // upper left world coordinate
    pos_y: 2390,
    scale: 5.54,

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
