import { LoadedMap } from "..";
import OfficialDefault from "./map_style_cs2.png";

export default {
    mapName: "de_basalt",
    displayName: "Basalt",

    pos_x: -2345.6, // upper left world coordinate
    pos_y: 2391.8,
    scale: 4.37,

    verticalSections: [
        {
            name: "default",
            altitudeMax: 10000,
            altitudeMin: -10000,
        }
    ],

    mapStyles: [
        {
            name: "Official",
            map: {
                default: OfficialDefault
            }
        }
    ]
} satisfies LoadedMap;
