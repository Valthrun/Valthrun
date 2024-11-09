import { LoadedMap } from "..";
import OfficialDefault from "./map_style_cs2.png";

export default {
    mapName: "de_thera",
    displayName: "Thera",

    pos_x: -85.609764, // upper left world coordinate
    pos_y: 2261.8025,
    scale: 4.85,

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
                default: OfficialDefault,
            }
        }
    ]
} satisfies LoadedMap;
