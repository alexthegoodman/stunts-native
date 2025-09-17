// #[derive(Debug, Clone)]
// pub struct AnimationIdea {
//     pub object_description: String,
//     pub position_description: String,
//     pub rotation_description: String,
//     pub scale_description: String,
//     pub opacity_description: String,
// }

// pub fn get_animation_ideas() -> Vec<AnimationIdea> {
//     vec![
//         // Modern UI/UX Animations
//         AnimationIdea {
//             object_description: "Sleek notification bell icon".to_string(),
//             position_description: "Gently bounces up and down with elastic motion".to_string(),
//             rotation_description: "Wiggles left and right like ringing".to_string(),
//             scale_description: "Pulses slightly larger on each bounce".to_string(),
//             opacity_description: "Stays fully visible throughout".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Modern loading spinner with gradient".to_string(),
//             position_description: "Remains centered and stationary".to_string(),
//             rotation_description: "Spins continuously clockwise at smooth speed".to_string(),
//             scale_description: "Maintains constant size".to_string(),
//             opacity_description: "Fades in smoothly, then stays visible".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Minimalist check mark icon".to_string(),
//             position_description: "Slides in from bottom right corner".to_string(),
//             rotation_description: "No rotation, stays upright".to_string(),
//             scale_description: "Grows from tiny to normal size with bounce".to_string(),
//             opacity_description: "Fades in as it appears".to_string(),
//         },
        
//         // Product Demo Essentials
//         AnimationIdea {
//             object_description: "Smartphone mockup with app interface".to_string(),
//             position_description: "Floats gently up and down".to_string(),
//             rotation_description: "Slowly rotates to show 3D depth".to_string(),
//             scale_description: "Slightly grows when highlighted".to_string(),
//             opacity_description: "Fully opaque with subtle glow effect".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Feature callout bubble with arrow".to_string(),
//             position_description: "Slides in from left side of screen".to_string(),
//             rotation_description: "No rotation".to_string(),
//             scale_description: "Pops in with elastic bounce effect".to_string(),
//             opacity_description: "Fades in smoothly then pulses visibility".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Dashboard chart with rising bars".to_string(),
//             position_description: "Charts rise from bottom baseline".to_string(),
//             rotation_description: "No rotation".to_string(),
//             scale_description: "Bars grow upward in sequence".to_string(),
//             opacity_description: "Each bar fades in as it grows".to_string(),
//         },
        
//         // Traditional/Classic Animations
//         AnimationIdea {
//             object_description: "Vintage pocket watch with chain".to_string(),
//             position_description: "Swings back and forth like pendulum".to_string(),
//             rotation_description: "Watch face rotates showing time passing".to_string(),
//             scale_description: "Stays same size throughout".to_string(),
//             opacity_description: "Fully visible with aged sepia tint".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Classic film reel with celluloid strips".to_string(),
//             position_description: "Remains stationary in center".to_string(),
//             rotation_description: "Spins like old movie projector".to_string(),
//             scale_description: "Maintains constant size".to_string(),
//             opacity_description: "Flickers slightly like old film".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Typewriter with visible keys".to_string(),
//             position_description: "Keys press down individually".to_string(),
//             rotation_description: "No rotation".to_string(),
//             scale_description: "Keys depress and return to normal".to_string(),
//             opacity_description: "Fully visible with typed letters appearing".to_string(),
//         },
        
//         // Esoteric/Abstract Animations
//         AnimationIdea {
//             object_description: "Geometric mandala with intricate patterns".to_string(),
//             position_description: "Slowly orbits around invisible center point".to_string(),
//             rotation_description: "Rotates on its own axis in opposite direction".to_string(),
//             scale_description: "Pulses between small and large rhythmically".to_string(),
//             opacity_description: "Fades in and out creating breathing effect".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Crystalline fractal structure".to_string(),
//             position_description: "Drifts diagonally across space".to_string(),
//             rotation_description: "Tumbles in multiple dimensions".to_string(),
//             scale_description: "Morphs size based on golden ratio".to_string(),
//             opacity_description: "Phases between translucent and solid".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Flowing particle cloud system".to_string(),
//             position_description: "Particles swirl in tornado formation".to_string(),
//             rotation_description: "Individual particles spin randomly".to_string(),
//             scale_description: "Particles grow and shrink organically".to_string(),
//             opacity_description: "Particles fade in and out like fireflies".to_string(),
//         },
        
//         // Business/Corporate Animations
//         AnimationIdea {
//             object_description: "Corporate logo with clean typography".to_string(),
//             position_description: "Enters from top with gentle drop".to_string(),
//             rotation_description: "No rotation, maintains brand integrity".to_string(),
//             scale_description: "Starts small and grows to final size".to_string(),
//             opacity_description: "Fades in professionally and smoothly".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Business card with contact details".to_string(),
//             position_description: "Flips in from right side".to_string(),
//             rotation_description: "Rotates to show front and back".to_string(),
//             scale_description: "Maintains professional proportions".to_string(),
//             opacity_description: "Solid opacity with subtle shadow".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Growth arrow pointing upward".to_string(),
//             position_description: "Moves steadily upward and forward".to_string(),
//             rotation_description: "No rotation, stays pointed up".to_string(),
//             scale_description: "Gets larger as it rises".to_string(),
//             opacity_description: "Bright and fully visible".to_string(),
//         },
        
//         // Social Media/Content Creator
//         AnimationIdea {
//             object_description: "Heart icon with sparkle effects".to_string(),
//             position_description: "Bounces playfully in center".to_string(),
//             rotation_description: "No rotation".to_string(),
//             scale_description: "Beats like real heart, larger then smaller".to_string(),
//             opacity_description: "Fully bright with sparkles fading in".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Subscribe button with play symbol".to_string(),
//             position_description: "Pulses in place with magnetic energy".to_string(),
//             rotation_description: "No rotation".to_string(),
//             scale_description: "Grows bigger on hover effect".to_string(),
//             opacity_description: "Bright red with white text, fully opaque".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Thumbs up emoji with motion lines".to_string(),
//             position_description: "Shoots up from bottom of screen".to_string(),
//             rotation_description: "Spins once as it rises".to_string(),
//             scale_description: "Starts tiny and grows to full size".to_string(),
//             opacity_description: "Fades in as it appears".to_string(),
//         },
        
//         // E-commerce/Retail
//         AnimationIdea {
//             object_description: "Shopping cart with items inside".to_string(),
//             position_description: "Rolls from left to right across screen".to_string(),
//             rotation_description: "Wheels spin as cart moves".to_string(),
//             scale_description: "Stays consistent size".to_string(),
//             opacity_description: "Fully visible throughout journey".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Product box with ribbon and bow".to_string(),
//             position_description: "Drops down from above like delivery".to_string(),
//             rotation_description: "Spins once while falling".to_string(),
//             scale_description: "Bounces slightly when it lands".to_string(),
//             opacity_description: "Solid and bright like new product".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Credit card with chip and stripe".to_string(),
//             position_description: "Slides horizontally like being swiped".to_string(),
//             rotation_description: "No rotation, stays flat".to_string(),
//             scale_description: "Maintains realistic proportions".to_string(),
//             opacity_description: "Fully opaque with slight metallic sheen".to_string(),
//         },
        
//         // Educational/Learning
//         AnimationIdea {
//             object_description: "Open book with fluttering pages".to_string(),
//             position_description: "Hovers gently with floating motion".to_string(),
//             rotation_description: "Pages flip individually showing content".to_string(),
//             scale_description: "Book stays same size".to_string(),
//             opacity_description: "Fully visible with pages semi-transparent".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Light bulb with glowing filament".to_string(),
//             position_description: "Bobs up and down like floating idea".to_string(),
//             rotation_description: "No rotation".to_string(),
//             scale_description: "Pulses brighter and dimmer rhythmically".to_string(),
//             opacity_description: "Glows brightly then dims in cycle".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Graduation cap with tassel".to_string(),
//             position_description: "Thrown up in celebration".to_string(),
//             rotation_description: "Spins as it flies through air".to_string(),
//             scale_description: "Gets smaller as it goes higher".to_string(),
//             opacity_description: "Starts bright, fades as it rises".to_string(),
//         },
        
//         // Tech/Gaming
//         AnimationIdea {
//             object_description: "Pixelated 8-bit coin spinning".to_string(),
//             position_description: "Bounces in arcade-style pattern".to_string(),
//             rotation_description: "Flips showing different sides".to_string(),
//             scale_description: "Maintains blocky pixel proportions".to_string(),
//             opacity_description: "Bright and solid like retro game".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Glowing neon circuit board pattern".to_string(),
//             position_description: "Electricity flows along pathways".to_string(),
//             rotation_description: "No rotation".to_string(),
//             scale_description: "Pulses grow along circuit traces".to_string(),
//             opacity_description: "Bright neon glow with dark background".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Holographic user interface panel".to_string(),
//             position_description: "Materializes from scattered particles".to_string(),
//             rotation_description: "Slowly rotates showing dimensionality".to_string(),
//             scale_description: "Assembles from small to full size".to_string(),
//             opacity_description: "Translucent with bright blue glow".to_string(),
//         },
        
//         // Nature/Organic
//         AnimationIdea {
//             object_description: "Leaf falling from invisible tree".to_string(),
//             position_description: "Drifts down in realistic zigzag pattern".to_string(),
//             rotation_description: "Tumbles naturally as it falls".to_string(),
//             scale_description: "Stays natural leaf size".to_string(),
//             opacity_description: "Fully visible with autumn coloring".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Butterfly with detailed wing patterns".to_string(),
//             position_description: "Flutters in figure-eight flight path".to_string(),
//             rotation_description: "Wings beat up and down rhythmically".to_string(),
//             scale_description: "Maintains natural butterfly proportions".to_string(),
//             opacity_description: "Vibrant colors, fully opaque".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Ocean wave with foam crest".to_string(),
//             position_description: "Rolls from left to right continuously".to_string(),
//             rotation_description: "Wave curls and crashes naturally".to_string(),
//             scale_description: "Grows taller as it approaches shore".to_string(),
//             opacity_description: "Blue water with white foam transparency".to_string(),
//         },
        
//         // Abstract/Artistic
//         AnimationIdea {
//             object_description: "Ink blot spreading on paper".to_string(),
//             position_description: "Expands outward from center point".to_string(),
//             rotation_description: "No rotation, organic spread".to_string(),
//             scale_description: "Grows from dot to full blot".to_string(),
//             opacity_description: "Starts transparent, becomes solid black".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Origami crane unfolding".to_string(),
//             position_description: "Stays centered while transforming".to_string(),
//             rotation_description: "Rotates to show folding process".to_string(),
//             scale_description: "Changes size as it unfolds".to_string(),
//             opacity_description: "Paper texture, fully visible".to_string(),
//         },
//         AnimationIdea {
//             object_description: "Smoke trail dissolving upward".to_string(),
//             position_description: "Rises and disperses into air".to_string(),
//             rotation_description: "Swirls and twists naturally".to_string(),
//             scale_description: "Spreads wider as it rises".to_string(),
//             opacity_description: "Fades from solid to transparent".to_string(),
//         },
//     ]
// }

#[derive(Debug, Clone)]
pub struct AnimationIdea {
    pub object_description: String,
    pub position_description: String,
    pub rotation_description: String,
    pub scale_description: String,
    pub opacity_description: String,
}

pub fn get_animation_ideas() -> Vec<AnimationIdea> {
    vec![
        // Subtle UI/UX Animations
        AnimationIdea {
            object_description: "Modern notification bell icon".to_string(),
            position_description: "Gently bobs up and down".to_string(),
            rotation_description: "Wiggles slightly back and forth".to_string(),
            scale_description: "Pulses just a bit larger occasionally".to_string(),
            opacity_description: "Stays fully visible".to_string(),
        },
        AnimationIdea {
            object_description: "Loading spinner with clean design".to_string(),
            position_description: "Remains perfectly centered".to_string(),
            rotation_description: "Spins smoothly and continuously".to_string(),
            scale_description: "Maintains steady size".to_string(),
            opacity_description: "Fades in smoothly".to_string(),
        },
        AnimationIdea {
            object_description: "Check mark icon".to_string(),
            position_description: "Settles into place".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Grows from small to normal with gentle bounce".to_string(),
            opacity_description: "Fades in".to_string(),
        },
        
        // Product Showcase Essentials
        AnimationIdea {
            object_description: "Smartphone in hand".to_string(),
            position_description: "Floats gently with subtle drift".to_string(),
            rotation_description: "Slowly tilts to show depth".to_string(),
            scale_description: "Stays consistent size".to_string(),
            opacity_description: "Fully visible with soft presence".to_string(),
        },
        AnimationIdea {
            object_description: "Laptop computer open".to_string(),
            position_description: "Hovers with barely noticeable movement".to_string(),
            rotation_description: "Rotates very slowly on lazy susan".to_string(),
            scale_description: "Maintains realistic proportions".to_string(),
            opacity_description: "Solid and clear".to_string(),
        },
        AnimationIdea {
            object_description: "Product feature callout".to_string(),
            position_description: "Drifts in gently".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Appears with soft growth".to_string(),
            opacity_description: "Fades in smoothly".to_string(),
        },
        
        // Simple Object Behaviors
        AnimationIdea {
            object_description: "Coffee mug on surface".to_string(),
            position_description: "Sits still with tiny natural tremor".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Stays same size".to_string(),
            opacity_description: "Fully opaque".to_string(),
        },
        AnimationIdea {
            object_description: "Book lying flat".to_string(),
            position_description: "Rests in place".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Maintains book proportions".to_string(),
            opacity_description: "Solid visibility".to_string(),
        },
        AnimationIdea {
            object_description: "Watch on wrist".to_string(),
            position_description: "Moves with gentle breathing motion".to_string(),
            rotation_description: "Stays oriented naturally".to_string(),
            scale_description: "Consistent watch size".to_string(),
            opacity_description: "Clear and visible".to_string(),
        },
        
        // Floating/Hovering Elements
        AnimationIdea {
            object_description: "Cloud icon".to_string(),
            position_description: "Drifts slowly across view".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Breathes slightly larger and smaller".to_string(),
            opacity_description: "Semi-transparent like real cloud".to_string(),
        },
        AnimationIdea {
            object_description: "Paper airplane".to_string(),
            position_description: "Glides smoothly forward".to_string(),
            rotation_description: "Banks gently left and right".to_string(),
            scale_description: "Stays same size".to_string(),
            opacity_description: "Fully visible".to_string(),
        },
        AnimationIdea {
            object_description: "Balloon with string".to_string(),
            position_description: "Bobs up and down softly".to_string(),
            rotation_description: "Sways just a little".to_string(),
            scale_description: "Maintains balloon size".to_string(),
            opacity_description: "Bright and opaque".to_string(),
        },
        
        // Product on Display
        AnimationIdea {
            object_description: "Sneaker on pedestal".to_string(),
            position_description: "Stays perfectly positioned".to_string(),
            rotation_description: "Turns slowly to show all angles".to_string(),
            scale_description: "Maintains shoe proportions".to_string(),
            opacity_description: "Fully visible".to_string(),
        },
        AnimationIdea {
            object_description: "Bottle of perfume".to_string(),
            position_description: "Sits elegantly in place".to_string(),
            rotation_description: "Rotates gently on display".to_string(),
            scale_description: "Stays realistic size".to_string(),
            opacity_description: "Glass transparency with solid label".to_string(),
        },
        AnimationIdea {
            object_description: "Headphones on stand".to_string(),
            position_description: "Rests naturally".to_string(),
            rotation_description: "Turns slowly to show design".to_string(),
            scale_description: "Keeps headphone proportions".to_string(),
            opacity_description: "Solid and clear".to_string(),
        },
        
        // Gentle Emphasis
        AnimationIdea {
            object_description: "Important text label".to_string(),
            position_description: "Settles into position".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Pulses subtly to draw attention".to_string(),
            opacity_description: "Fades in".to_string(),
        },
        AnimationIdea {
            object_description: "Arrow pointing to feature".to_string(),
            position_description: "Points steadily at target".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Stays consistent size".to_string(),
            opacity_description: "Bright and clear".to_string(),
        },
        AnimationIdea {
            object_description: "Highlight circle around object".to_string(),
            position_description: "Stays centered on target".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Pulses gently larger and smaller".to_string(),
            opacity_description: "Semi-transparent glow".to_string(),
        },
        
        // Natural Settling Motions
        AnimationIdea {
            object_description: "Coin landing on surface".to_string(),
            position_description: "Drops down and settles".to_string(),
            rotation_description: "Spins as it falls, then stops".to_string(),
            scale_description: "Maintains coin size".to_string(),
            opacity_description: "Metallic shine, fully visible".to_string(),
        },
        AnimationIdea {
            object_description: "Dice after being rolled".to_string(),
            position_description: "Comes to rest naturally".to_string(),
            rotation_description: "Settles to show final number".to_string(),
            scale_description: "Stays dice-sized".to_string(),
            opacity_description: "Solid white with dark dots".to_string(),
        },
        AnimationIdea {
            object_description: "Pen dropping onto desk".to_string(),
            position_description: "Falls and comes to rest".to_string(),
            rotation_description: "Rolls slightly then stops".to_string(),
            scale_description: "Realistic pen proportions".to_string(),
            opacity_description: "Fully opaque".to_string(),
        },
        
        // Breathing/Alive Elements
        AnimationIdea {
            object_description: "Heart shape".to_string(),
            position_description: "Stays centered".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Beats rhythmically like real heart".to_string(),
            opacity_description: "Warm red, fully visible".to_string(),
        },
        AnimationIdea {
            object_description: "Plant in pot".to_string(),
            position_description: "Sways very gently".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Stays natural plant size".to_string(),
            opacity_description: "Green and vibrant".to_string(),
        },
        AnimationIdea {
            object_description: "Candle flame".to_string(),
            position_description: "Flickers up and down subtly".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Dances between small and normal".to_string(),
            opacity_description: "Bright orange glow".to_string(),
        },
        
        // Tech Product Behaviors
        AnimationIdea {
            object_description: "Router with blinking lights".to_string(),
            position_description: "Sits stationary".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Maintains router size".to_string(),
            opacity_description: "Lights pulse on and off".to_string(),
        },
        AnimationIdea {
            object_description: "Charging cable connected".to_string(),
            position_description: "Lies naturally in place".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Keeps cable proportions".to_string(),
            opacity_description: "Fully visible".to_string(),
        },
        AnimationIdea {
            object_description: "Smart speaker cylinder".to_string(),
            position_description: "Sits perfectly still".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Pulses very slightly when active".to_string(),
            opacity_description: "Solid with subtle LED ring".to_string(),
        },
        
        // Simple Geometric Shapes
        AnimationIdea {
            object_description: "Sphere floating in space".to_string(),
            position_description: "Hovers with gentle drift".to_string(),
            rotation_description: "Spins slowly on axis".to_string(),
            scale_description: "Breathes subtly larger and smaller".to_string(),
            opacity_description: "Semi-transparent with inner glow".to_string(),
        },
        AnimationIdea {
            object_description: "Cube sitting on surface".to_string(),
            position_description: "Rests solidly in place".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Maintains cube proportions".to_string(),
            opacity_description: "Solid color, fully opaque".to_string(),
        },
        AnimationIdea {
            object_description: "Ring spinning in air".to_string(),
            position_description: "Floats steadily".to_string(),
            rotation_description: "Rotates smoothly around center".to_string(),
            scale_description: "Stays same ring size".to_string(),
            opacity_description: "Metallic finish, fully visible".to_string(),
        },
    ]
}