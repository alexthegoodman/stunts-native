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
        // Modern UI/UX Animations
        AnimationIdea {
            object_description: "Sleek notification bell icon".to_string(),
            position_description: "Gently bounces up and down with elastic motion".to_string(),
            rotation_description: "Wiggles left and right like ringing".to_string(),
            scale_description: "Pulses slightly larger on each bounce".to_string(),
            opacity_description: "Stays fully visible throughout".to_string(),
        },
        AnimationIdea {
            object_description: "Modern loading spinner with gradient".to_string(),
            position_description: "Remains centered and stationary".to_string(),
            rotation_description: "Spins continuously clockwise at smooth speed".to_string(),
            scale_description: "Maintains constant size".to_string(),
            opacity_description: "Fades in smoothly, then stays visible".to_string(),
        },
        AnimationIdea {
            object_description: "Minimalist check mark icon".to_string(),
            position_description: "Slides in from bottom right corner".to_string(),
            rotation_description: "No rotation, stays upright".to_string(),
            scale_description: "Grows from tiny to normal size with bounce".to_string(),
            opacity_description: "Fades in as it appears".to_string(),
        },
        
        // Product Demo Essentials
        AnimationIdea {
            object_description: "Smartphone mockup with app interface".to_string(),
            position_description: "Floats gently up and down".to_string(),
            rotation_description: "Slowly rotates to show 3D depth".to_string(),
            scale_description: "Slightly grows when highlighted".to_string(),
            opacity_description: "Fully opaque with subtle glow effect".to_string(),
        },
        AnimationIdea {
            object_description: "Feature callout bubble with arrow".to_string(),
            position_description: "Slides in from left side of screen".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Pops in with elastic bounce effect".to_string(),
            opacity_description: "Fades in smoothly then pulses visibility".to_string(),
        },
        AnimationIdea {
            object_description: "Dashboard chart with rising bars".to_string(),
            position_description: "Charts rise from bottom baseline".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Bars grow upward in sequence".to_string(),
            opacity_description: "Each bar fades in as it grows".to_string(),
        },
        
        // Traditional/Classic Animations
        AnimationIdea {
            object_description: "Vintage pocket watch with chain".to_string(),
            position_description: "Swings back and forth like pendulum".to_string(),
            rotation_description: "Watch face rotates showing time passing".to_string(),
            scale_description: "Stays same size throughout".to_string(),
            opacity_description: "Fully visible with aged sepia tint".to_string(),
        },
        AnimationIdea {
            object_description: "Classic film reel with celluloid strips".to_string(),
            position_description: "Remains stationary in center".to_string(),
            rotation_description: "Spins like old movie projector".to_string(),
            scale_description: "Maintains constant size".to_string(),
            opacity_description: "Flickers slightly like old film".to_string(),
        },
        AnimationIdea {
            object_description: "Typewriter with visible keys".to_string(),
            position_description: "Keys press down individually".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Keys depress and return to normal".to_string(),
            opacity_description: "Fully visible with typed letters appearing".to_string(),
        },
        
        // Esoteric/Abstract Animations
        AnimationIdea {
            object_description: "Geometric mandala with intricate patterns".to_string(),
            position_description: "Slowly orbits around invisible center point".to_string(),
            rotation_description: "Rotates on its own axis in opposite direction".to_string(),
            scale_description: "Pulses between small and large rhythmically".to_string(),
            opacity_description: "Fades in and out creating breathing effect".to_string(),
        },
        AnimationIdea {
            object_description: "Crystalline fractal structure".to_string(),
            position_description: "Drifts diagonally across space".to_string(),
            rotation_description: "Tumbles in multiple dimensions".to_string(),
            scale_description: "Morphs size based on golden ratio".to_string(),
            opacity_description: "Phases between translucent and solid".to_string(),
        },
        AnimationIdea {
            object_description: "Flowing particle cloud system".to_string(),
            position_description: "Particles swirl in tornado formation".to_string(),
            rotation_description: "Individual particles spin randomly".to_string(),
            scale_description: "Particles grow and shrink organically".to_string(),
            opacity_description: "Particles fade in and out like fireflies".to_string(),
        },
        
        // Business/Corporate Animations
        AnimationIdea {
            object_description: "Corporate logo with clean typography".to_string(),
            position_description: "Enters from top with gentle drop".to_string(),
            rotation_description: "No rotation, maintains brand integrity".to_string(),
            scale_description: "Starts small and grows to final size".to_string(),
            opacity_description: "Fades in professionally and smoothly".to_string(),
        },
        AnimationIdea {
            object_description: "Business card with contact details".to_string(),
            position_description: "Flips in from right side".to_string(),
            rotation_description: "Rotates to show front and back".to_string(),
            scale_description: "Maintains professional proportions".to_string(),
            opacity_description: "Solid opacity with subtle shadow".to_string(),
        },
        AnimationIdea {
            object_description: "Growth arrow pointing upward".to_string(),
            position_description: "Moves steadily upward and forward".to_string(),
            rotation_description: "No rotation, stays pointed up".to_string(),
            scale_description: "Gets larger as it rises".to_string(),
            opacity_description: "Bright and fully visible".to_string(),
        },
        
        // Social Media/Content Creator
        AnimationIdea {
            object_description: "Heart icon with sparkle effects".to_string(),
            position_description: "Bounces playfully in center".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Beats like real heart, larger then smaller".to_string(),
            opacity_description: "Fully bright with sparkles fading in".to_string(),
        },
        AnimationIdea {
            object_description: "Subscribe button with play symbol".to_string(),
            position_description: "Pulses in place with magnetic energy".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Grows bigger on hover effect".to_string(),
            opacity_description: "Bright red with white text, fully opaque".to_string(),
        },
        AnimationIdea {
            object_description: "Thumbs up emoji with motion lines".to_string(),
            position_description: "Shoots up from bottom of screen".to_string(),
            rotation_description: "Spins once as it rises".to_string(),
            scale_description: "Starts tiny and grows to full size".to_string(),
            opacity_description: "Fades in as it appears".to_string(),
        },
        
        // E-commerce/Retail
        AnimationIdea {
            object_description: "Shopping cart with items inside".to_string(),
            position_description: "Rolls from left to right across screen".to_string(),
            rotation_description: "Wheels spin as cart moves".to_string(),
            scale_description: "Stays consistent size".to_string(),
            opacity_description: "Fully visible throughout journey".to_string(),
        },
        AnimationIdea {
            object_description: "Product box with ribbon and bow".to_string(),
            position_description: "Drops down from above like delivery".to_string(),
            rotation_description: "Spins once while falling".to_string(),
            scale_description: "Bounces slightly when it lands".to_string(),
            opacity_description: "Solid and bright like new product".to_string(),
        },
        AnimationIdea {
            object_description: "Credit card with chip and stripe".to_string(),
            position_description: "Slides horizontally like being swiped".to_string(),
            rotation_description: "No rotation, stays flat".to_string(),
            scale_description: "Maintains realistic proportions".to_string(),
            opacity_description: "Fully opaque with slight metallic sheen".to_string(),
        },
        
        // Educational/Learning
        AnimationIdea {
            object_description: "Open book with fluttering pages".to_string(),
            position_description: "Hovers gently with floating motion".to_string(),
            rotation_description: "Pages flip individually showing content".to_string(),
            scale_description: "Book stays same size".to_string(),
            opacity_description: "Fully visible with pages semi-transparent".to_string(),
        },
        AnimationIdea {
            object_description: "Light bulb with glowing filament".to_string(),
            position_description: "Bobs up and down like floating idea".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Pulses brighter and dimmer rhythmically".to_string(),
            opacity_description: "Glows brightly then dims in cycle".to_string(),
        },
        AnimationIdea {
            object_description: "Graduation cap with tassel".to_string(),
            position_description: "Thrown up in celebration".to_string(),
            rotation_description: "Spins as it flies through air".to_string(),
            scale_description: "Gets smaller as it goes higher".to_string(),
            opacity_description: "Starts bright, fades as it rises".to_string(),
        },
        
        // Tech/Gaming
        AnimationIdea {
            object_description: "Pixelated 8-bit coin spinning".to_string(),
            position_description: "Bounces in arcade-style pattern".to_string(),
            rotation_description: "Flips showing different sides".to_string(),
            scale_description: "Maintains blocky pixel proportions".to_string(),
            opacity_description: "Bright and solid like retro game".to_string(),
        },
        AnimationIdea {
            object_description: "Glowing neon circuit board pattern".to_string(),
            position_description: "Electricity flows along pathways".to_string(),
            rotation_description: "No rotation".to_string(),
            scale_description: "Pulses grow along circuit traces".to_string(),
            opacity_description: "Bright neon glow with dark background".to_string(),
        },
        AnimationIdea {
            object_description: "Holographic user interface panel".to_string(),
            position_description: "Materializes from scattered particles".to_string(),
            rotation_description: "Slowly rotates showing dimensionality".to_string(),
            scale_description: "Assembles from small to full size".to_string(),
            opacity_description: "Translucent with bright blue glow".to_string(),
        },
        
        // Nature/Organic
        AnimationIdea {
            object_description: "Leaf falling from invisible tree".to_string(),
            position_description: "Drifts down in realistic zigzag pattern".to_string(),
            rotation_description: "Tumbles naturally as it falls".to_string(),
            scale_description: "Stays natural leaf size".to_string(),
            opacity_description: "Fully visible with autumn coloring".to_string(),
        },
        AnimationIdea {
            object_description: "Butterfly with detailed wing patterns".to_string(),
            position_description: "Flutters in figure-eight flight path".to_string(),
            rotation_description: "Wings beat up and down rhythmically".to_string(),
            scale_description: "Maintains natural butterfly proportions".to_string(),
            opacity_description: "Vibrant colors, fully opaque".to_string(),
        },
        AnimationIdea {
            object_description: "Ocean wave with foam crest".to_string(),
            position_description: "Rolls from left to right continuously".to_string(),
            rotation_description: "Wave curls and crashes naturally".to_string(),
            scale_description: "Grows taller as it approaches shore".to_string(),
            opacity_description: "Blue water with white foam transparency".to_string(),
        },
        
        // Abstract/Artistic
        AnimationIdea {
            object_description: "Ink blot spreading on paper".to_string(),
            position_description: "Expands outward from center point".to_string(),
            rotation_description: "No rotation, organic spread".to_string(),
            scale_description: "Grows from dot to full blot".to_string(),
            opacity_description: "Starts transparent, becomes solid black".to_string(),
        },
        AnimationIdea {
            object_description: "Origami crane unfolding".to_string(),
            position_description: "Stays centered while transforming".to_string(),
            rotation_description: "Rotates to show folding process".to_string(),
            scale_description: "Changes size as it unfolds".to_string(),
            opacity_description: "Paper texture, fully visible".to_string(),
        },
        AnimationIdea {
            object_description: "Smoke trail dissolving upward".to_string(),
            position_description: "Rises and disperses into air".to_string(),
            rotation_description: "Swirls and twists naturally".to_string(),
            scale_description: "Spreads wider as it rises".to_string(),
            opacity_description: "Fades from solid to transparent".to_string(),
        },
    ]
}