//! Sample tutorial and shortcut tables of the help screen (placeholder data, in Spanish).

use crate::features::help::models::{Emphasis, Shortcut, TutorialStep};
use Emphasis::{Key, Plain, Strong};

/// The tutorial steps.
pub fn steps() -> Vec<TutorialStep> {
    vec![
        TutorialStep {
            topic: "Visor CAD en tiempo real",
            title: "Navegar por las vistas 3D",
            body: vec![
                ("Mantén ", Plain),
                ("RMB", Key),
                (" para orbitar alrededor de ", Plain),
                ("Villa Moderna L26", Strong),
                (" y usa las teclas ", Plain),
                ("WASD", Key),
                (" junto con ", Plain),
                ("Q", Key),
                (" y ", Plain),
                ("E", Key),
                (
                    " para volar con fluidez por la estructura de concreto y vidrio a 37 FPS.",
                    Plain,
                ),
            ],
            caption: "Cámara de vuelo libre: eje de la piscina",
            tiles: [
                ("Órbita con RMB", "Acimut e inclinación libres"),
                ("Control de vuelo WASD", "Traslación planar"),
                ("Teclas de elevación Q/E", "Desplazamiento en el eje Z"),
                ("Cuadrícula métrica", "Subdivisiones de 1.00m"),
            ],
        },
        TutorialStep {
            topic: "Colocación de objetos",
            title: "Colocar vegetación en la escena",
            body: vec![
                ("Abre la ", Plain),
                ("Biblioteca", Strong),
                (", elige un ejemplar y pulsa ", Plain),
                ("Colocar en escena", Key),
                (". Usa ", Plain),
                ("Espacio + clic", Key),
                (" para estampar varias instancias.", Plain),
            ],
            caption: "Modo construcción: borde del dosel",
            tiles: [
                ("Alinear al terreno", "Tecla F"),
                ("Variación aleatoria", "Tecla R"),
                ("Imán de vértices", "Ajuste al muro"),
                ("Contador de instancias", "Paso a paso"),
            ],
        },
        TutorialStep {
            topic: "Materiales",
            title: "Editar materiales PBR",
            body: vec![
                ("Selecciona una superficie y ajusta ", Plain),
                ("rugosidad", Key),
                (", ", Plain),
                ("metálico", Key),
                (" y ", Plain),
                ("especular", Key),
                (" en el editor de materiales.", Plain),
            ],
            caption: "Esfera de sombreado: concreto arquitectónico",
            tiles: [
                ("Mapa de rugosidad", "Textura EXR de 16 bits"),
                ("Pátina de intemperie", "Simulación ambiental"),
                ("Canales PBR", "Normal, metálico, especular"),
                ("Aplicar material", "Sincroniza la caché"),
            ],
        },
        TutorialStep {
            topic: "Película",
            title: "Animar una cámara",
            body: vec![
                ("Añade ", Plain),
                ("fotogramas clave", Key),
                (" en la línea de tiempo y ajusta la ", Plain),
                ("curva Bézier", Strong),
                (" para suavizar el recorrido.", Plain),
            ],
            caption: "Órbita cinematográfica con dron",
            tiles: [
                ("Añadir fotograma", "Posición del cabezal"),
                ("Auto-spline", "Curvas suaves"),
                ("Pistas", "Cámara, altura, focal, sol"),
                ("Renderizar clip", "Cola de exportación"),
            ],
        },
        TutorialStep {
            topic: "Render",
            title: "Preparar un render",
            body: vec![
                ("Elige un ", Plain),
                ("perfil de render", Strong),
                (" en Ajustes y revisa la ", Plain),
                ("telemetría", Key),
                (" antes de lanzar el trabajo.", Plain),
            ],
            caption: "Perfil: producción de estudio",
            tiles: [
                ("Calidad de rayos", "Niveles bajo a ultra"),
                ("Escala de resolución", "50% a 100%"),
                ("Perfiles", "Borrador, interactivo, producción"),
                ("Telemetría", "VRAM y temperatura"),
            ],
        },
        TutorialStep {
            topic: "Proyectos",
            title: "Guardar y exportar",
            body: vec![
                ("Desde ", Plain),
                ("Proyectos", Strong),
                (
                    " puedes cargar escenas, revisar la cola y exportar por lotes con ",
                    Plain,
                ),
                ("Exportar todo", Key),
                (".", Plain),
            ],
            caption: "Repositorio de trabajo",
            tiles: [
                ("Cargar proyecto", "Abre la vista 3D"),
                ("Cola de render", "Trabajos pendientes"),
                ("Lotes", "Preajuste de destino"),
                ("Archivados", "Escenas antiguas"),
            ],
        },
    ]
}

/// Shortcuts of the asset placement group.
pub fn placement_shortcuts() -> Vec<Shortcut> {
    vec![
        Shortcut {
            action: "Alinear a la normal del terreno",
            keys: "F",
        },
        Shortcut {
            action: "Aleatorizar giro y escala",
            keys: "R",
        },
        Shortcut {
            action: "Estampar instancias",
            keys: "Espacio + clic",
        },
        Shortcut {
            action: "Ajustar a la malla más cercana",
            keys: "V",
        },
    ]
}

/// Shortcuts of the transform and selection group.
pub fn transform_shortcuts() -> Vec<Shortcut> {
    vec![
        Shortcut {
            action: "Añadir varios objetos",
            keys: "Ctrl + clic",
        },
        Shortcut {
            action: "Clonar interactivo",
            keys: "Alt + arrastrar",
        },
        Shortcut {
            action: "Alternar gizmo (W, E, R)",
            keys: "G",
        },
        Shortcut {
            action: "Aislar nodos seleccionados",
            keys: "I",
        },
    ]
}

/// Shortcuts handled by the application shell.
pub fn app_shortcuts() -> Vec<Shortcut> {
    vec![
        Shortcut {
            action: "Nueva escena",
            keys: "Ctrl + N",
        },
        Shortcut {
            action: "Abrir proyecto",
            keys: "Ctrl + O",
        },
        Shortcut {
            action: "Guardar proyecto",
            keys: "Ctrl + S",
        },
        Shortcut {
            action: "Exportar imagen",
            keys: "Ctrl + E",
        },
        Shortcut {
            action: "Preferencias",
            keys: "Ctrl + ,",
        },
        Shortcut {
            action: "Atajos de teclado",
            keys: "Ctrl + /",
        },
        Shortcut {
            action: "Cerrar diálogo",
            keys: "Esc",
        },
    ]
}

/// Navigation shortcuts of the 3D view.
pub fn navigation_shortcuts() -> Vec<Shortcut> {
    vec![
        Shortcut {
            action: "Orbitar",
            keys: "RMB",
        },
        Shortcut {
            action: "Volar adelante / atrás",
            keys: "W / S",
        },
        Shortcut {
            action: "Desplazarse a los lados",
            keys: "A / D",
        },
        Shortcut {
            action: "Subir / bajar",
            keys: "E / Q",
        },
        Shortcut {
            action: "Sprint x4",
            keys: "Mayús",
        },
    ]
}
