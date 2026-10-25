"""Model the device icons in Blender and render each one turning a full circle.

Run headless, one model per call:

    blender --background --factory-startup --python render_models.py -- \
        --model airpod_right --out <dir> [--angles 0,90,180,270] [--size 160]         [--airpods-fbx <AirPods 4.fbx>] [--iphone-glb <iphone.glb> --iphone-textures <dir>]

Every frame is a transparent PNG named by its index. `build_sprites.py` joins
them into the sprite strips the widget plays; frame 0 and the last frame both
show the front.

Without the model files the devices are built from primitives here. With them,
the downloaded AirPods 4 (FBX) and iPhone 14 Pro (glTF converted from the
Sketchfab COLLADA, plus its PBR texture folder) are rendered instead.
"""
import argparse
import math
import sys
from pathlib import Path

import bmesh
import bpy
from mathutils import Euler, Matrix, Quaternion, Vector

# 96 steps play at 60 fps over the widget's 1.6 s turn. The easing is baked
# into the angles, so the widget can step through the frames evenly.
FULL_TURN_FRAMES = 96
CAMERA_ELEVATION_DEG = 14
CAMERA_DISTANCE = 30.0
FRAME_MARGIN = 1.08
# Every model's front view covers the same share of its frame, measured as the
# square root of its silhouette's width x height. Matching only the height
# would make the squat case look far bigger than the slim pods.
FRONT_VIEW_FILL = 0.52
SAMPLES = 96

WHITE_PLASTIC = (0.93, 0.93, 0.92, 1)
GRILLE = (0.02, 0.02, 0.022, 1)


# --- materials ---------------------------------------------------------------

def principled(name, color, roughness, metallic=0.0, coat=0.0, emission=None, emission_strength=0.0):
    material = bpy.data.materials.new(name)
    material.use_nodes = True
    bsdf = material.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = color
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metallic
    bsdf.inputs["Coat Weight"].default_value = coat
    if emission:
        bsdf.inputs["Emission Color"].default_value = emission
        bsdf.inputs["Emission Strength"].default_value = emission_strength
    return material


def plastic():
    return principled("plastic", WHITE_PLASTIC, roughness=0.32, coat=0.35)


def case_plastic(seam_z, seam_width):
    """White plastic with a thin dark band where lid and body meet."""
    material = plastic()
    material.name = "case_plastic"
    nodes, links = material.node_tree.nodes, material.node_tree.links
    coords = nodes.new("ShaderNodeTexCoord")
    separate = nodes.new("ShaderNodeSeparateXYZ")
    distance = nodes.new("ShaderNodeMath")
    distance.operation = "ABSOLUTE"
    offset = nodes.new("ShaderNodeMath")
    offset.operation = "SUBTRACT"
    offset.inputs[1].default_value = seam_z
    ramp = nodes.new("ShaderNodeValToRGB")
    ramp.color_ramp.interpolation = "EASE"
    ramp.color_ramp.elements[0].position = seam_width * 0.5
    ramp.color_ramp.elements[0].color = (0.35, 0.35, 0.35, 1)
    ramp.color_ramp.elements[1].position = seam_width
    ramp.color_ramp.elements[1].color = WHITE_PLASTIC
    links.new(coords.outputs["Object"], separate.inputs[0])
    links.new(separate.outputs["Z"], offset.inputs[0])
    links.new(offset.outputs[0], distance.inputs[0])
    links.new(distance.outputs[0], ramp.inputs["Fac"])
    links.new(ramp.outputs["Color"], nodes["Principled BSDF"].inputs["Base Color"])
    return material


def screen_material():
    """Lit wallpaper: deep blue at the bottom rising into violet, close to the old CSS phone."""
    material = principled("screen", (0, 0, 0, 1), roughness=0.12, coat=0.4)
    nodes, links = material.node_tree.nodes, material.node_tree.links
    coords = nodes.new("ShaderNodeTexCoord")
    separate = nodes.new("ShaderNodeSeparateXYZ")
    height = nodes.new("ShaderNodeMapRange")
    height.inputs["From Min"].default_value = -7.0
    height.inputs["From Max"].default_value = 7.0
    ramp = nodes.new("ShaderNodeValToRGB")
    ramp.color_ramp.interpolation = "EASE"
    ramp.color_ramp.elements[0].color = (0.02, 0.03, 0.18, 1)
    ramp.color_ramp.elements[1].position = 0.55
    ramp.color_ramp.elements[1].color = (0.1, 0.14, 0.62, 1)
    top = ramp.color_ramp.elements.new(1.0)
    top.color = (0.5, 0.18, 0.75, 1)
    links.new(coords.outputs["Object"], separate.inputs[0])
    links.new(separate.outputs["Z"], height.inputs["Value"])
    links.new(height.outputs["Result"], ramp.inputs["Fac"])
    bsdf = nodes["Principled BSDF"]
    links.new(ramp.outputs["Color"], bsdf.inputs["Emission Color"])
    bsdf.inputs["Emission Strength"].default_value = 1.1
    return material


# --- geometry helpers --------------------------------------------------------

def link(obj):
    bpy.context.scene.collection.objects.link(obj)
    return obj


def mesh_object(name, mesh, material, smooth=True):
    obj = link(bpy.data.objects.new(name, mesh))
    obj.data.materials.append(material)
    if smooth:
        for polygon in obj.data.polygons:
            polygon.use_smooth = True
    return obj


def metaball_mesh(name, elements, resolution=0.03):
    """Blends ellipsoids and capsules into one smooth surface, then freezes it to a mesh."""
    meta = bpy.data.metaballs.new(name)
    meta.resolution = meta.render_resolution = resolution
    meta.threshold = 0.6
    for kind, co, radius, size, rotation in elements:
        element = meta.elements.new(type=kind)
        element.co, element.radius = co, radius
        element.size_x, element.size_y, element.size_z = size
        element.rotation = rotation
    obj = link(bpy.data.objects.new(name + "_meta", meta))
    depsgraph = bpy.context.evaluated_depsgraph_get()
    mesh = bpy.data.meshes.new_from_object(obj.evaluated_get(depsgraph))
    bpy.data.objects.remove(obj)
    return mesh


def ellipsoid(name, location, scale, material, rotation=(0, 0, 0)):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=48, ring_count=24, location=location, rotation=rotation)
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = scale
    obj.data.materials.append(material)
    bpy.ops.object.shade_smooth()
    return obj


def rounded_rect_mesh(name, width, height, radius, depth, corner_segments=12):
    """A slab whose outline is a rounded rectangle, standing in the XZ plane, `depth` thick along Y."""
    bm = bmesh.new()
    verts = []
    corners = [(width / 2 - radius, height / 2 - radius, 0), (-width / 2 + radius, height / 2 - radius, 90),
               (-width / 2 + radius, -height / 2 + radius, 180), (width / 2 - radius, -height / 2 + radius, 270)]
    for cx, cz, start in corners:
        for step in range(corner_segments + 1):
            angle = math.radians(start + 90 * step / corner_segments)
            verts.append(bm.verts.new((cx + radius * math.cos(angle), -depth / 2, cz + radius * math.sin(angle))))
    face = bm.faces.new(verts)
    extruded = bmesh.ops.extrude_face_region(bm, geom=[face])
    moved = [v for v in extruded["geom"] if isinstance(v, bmesh.types.BMVert)]
    bmesh.ops.translate(bm, vec=(0, depth, 0), verts=moved)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    return mesh


def add_bevel(obj, width, segments, angle_deg=40):
    bevel = obj.modifiers.new("bevel", "BEVEL")
    bevel.width, bevel.segments = width, segments
    bevel.limit_method = "ANGLE"
    bevel.angle_limit = math.radians(angle_deg)
    bevel.harden_normals = False
    return bevel


def parent_all(root_name, objects):
    root = link(bpy.data.objects.new(root_name, None))
    for obj in objects:
        obj.parent = root
    return root


# --- models ------------------------------------------------------------------

def paint_regions(obj, regions):
    """Assigns a material to the faces inside ellipsoidal regions, so details sit flush in the surface."""
    for material, center, radii in regions:
        obj.data.materials.append(material)
        index = len(obj.data.materials) - 1
        for polygon in obj.data.polygons:
            offset = polygon.center - Vector(center)
            if sum((offset[axis] / radii[axis]) ** 2 for axis in range(3)) <= 1:
                polygon.material_index = index


def build_airpod(is_left):
    """AirPods 4: rounded bud on top, short flat oval stem below, dark grille facing the ear."""
    capsule_upright = Quaternion((0, 1, 0), math.radians(90))
    no_rotation = Quaternion()
    mesh = metaball_mesh("airpod", [
        ("ELLIPSOID", (0, 0, 0), 1.15, (1.0, 0.9, 0.88), no_rotation),
        ("CAPSULE", (0.2, 0.14, -1.15), 0.44, (0.62, 0.56, 0.56), capsule_upright),
    ], resolution=0.014)
    body = mesh_object("airpod_body", mesh, plastic())
    grille_material = principled("grille", GRILLE, roughness=0.6)
    paint_regions(body, [
        (grille_material, (-0.95, -0.25, 0.12), (0.42, 0.44, 0.36)),
        (grille_material, (-0.2, -0.95, 0.45), (0.3, 0.3, 0.13)),
        (principled("tip", (0.55, 0.55, 0.57, 1), 0.3, metallic=1.0), (0.2, 0.14, -2.05), (0.5, 0.5, 0.12)),
    ])
    root = parent_all("airpod", [body])
    root.rotation_euler = Euler((0, math.radians(-8), 0))
    if is_left:
        root.scale.x = -1
    return root


def build_case():
    """AirPods 4 case: a soft rounded square, lid seam near the top, status light in front."""
    width, height, depth = 4.62, 5.01, 2.12
    seam_z = height / 2 - 1.05
    mesh = rounded_rect_mesh("case", width, height, 1.35, depth, corner_segments=16)
    body = mesh_object("case_body", mesh, case_plastic(seam_z, 0.035))
    add_bevel(body, width=0.85, segments=10)
    body.modifiers.new("subsurf", "SUBSURF").levels = 2
    light = ellipsoid("status_light", (0, -depth / 2 + 0.02, seam_z - 0.75), (0.05, 0.02, 0.05),
                      principled("light", (0.1, 0.9, 0.3, 1), 0.3, emission=(0.1, 1.0, 0.3, 1), emission_strength=4))
    return parent_all("case", [body, light])


def build_iphone():
    """A current iPhone: titanium frame, lit screen with the island, camera plateau on the back."""
    width, height, depth = 7.16, 14.76, 0.8
    titanium = principled("titanium", (0.62, 0.62, 0.64, 1), roughness=0.28, metallic=1.0)
    frame = mesh_object("frame", rounded_rect_mesh("frame", width, height, 1.05, depth, 16), titanium)
    add_bevel(frame, width=0.09, segments=4)

    glass_black = principled("bezel", (0.005, 0.005, 0.006, 1), roughness=0.05, coat=1.0)
    front = mesh_object("front", rounded_rect_mesh("front", width - 0.12, height - 0.12, 1.0, 0.02, 16), glass_black)
    front.location.y = -depth / 2 - 0.005
    screen = mesh_object("screen", rounded_rect_mesh("screen", width - 0.5, height - 0.5, 0.82, 0.02, 16), screen_material())
    screen.location.y = -depth / 2 - 0.012
    island = mesh_object("island", rounded_rect_mesh("island", 1.9, 0.56, 0.28, 0.02, 10), glass_black)
    island.location = (0, -depth / 2 - 0.02, height / 2 - 0.85)

    back_glass = principled("back", (0.55, 0.56, 0.6, 1), roughness=0.45, coat=0.4)
    back = mesh_object("back", rounded_rect_mesh("back", width - 0.12, height - 0.12, 1.0, 0.02, 16), back_glass)
    back.location.y = depth / 2 + 0.005
    plateau = mesh_object("plateau", rounded_rect_mesh("plateau", 3.0, 3.0, 0.75, 0.12, 12), back_glass)
    plateau.location = (-width / 2 + 1.9, depth / 2 + 0.06, height / 2 - 1.9)
    add_bevel(plateau, width=0.05, segments=3)

    lens_glass = principled("lens", (0.01, 0.01, 0.012, 1), roughness=0.05, coat=1.0)
    lens_ring = principled("lens_ring", (0.5, 0.5, 0.52, 1), roughness=0.25, metallic=1.0)
    parts = [frame, front, screen, island, back, plateau]
    for dx, dz in ((-0.62, 0.62), (-0.62, -0.62), (0.68, 0.0)):
        bpy.ops.mesh.primitive_cylinder_add(vertices=48, radius=0.55, depth=0.16, rotation=(math.radians(90), 0, 0))
        ring = bpy.context.active_object
        ring.location = (plateau.location.x + dx, depth / 2 + 0.17, plateau.location.z + dz)
        ring.data.materials.append(lens_ring)
        bpy.ops.mesh.primitive_cylinder_add(vertices=48, radius=0.42, depth=0.17, rotation=(math.radians(90), 0, 0))
        lens = bpy.context.active_object
        lens.location = ring.location
        lens.data.materials.append(lens_glass)
        parts += [ring, lens]
    return parent_all("iphone", parts)


# --- imported models ---------------------------------------------------------

# Group empties in the AirPods 4 FBX. Its object names are scrambled, so these
# are pinned to that one file.
AIRPODS_FBX_GROUPS = {"airpod_left": "oTgukKFHsbAaGBa", "airpod_right": "MSpBhwzeYpJKGDV", "airpods_case": "PxcMCoFCCTCdsTC"}
AIRPODS_FBX_LID = "nJoEOlgGwcdXBHJ"
AIRPODS_FBX_CASE_BODY = "iqLXttIronNPiXb"


def world_bounds(objects):
    corners = [obj.matrix_world @ Vector(corner) for obj in objects if obj.type == "MESH" for corner in obj.bound_box]
    low = Vector((min(c.x for c in corners), min(c.y for c in corners), min(c.z for c in corners)))
    high = Vector((max(c.x for c in corners), max(c.y for c in corners), max(c.z for c in corners)))
    return low, high


def wrap_scene(name):
    """Parents every top-level object to one root at the origin, so the model turns as a whole."""
    root = link(bpy.data.objects.new(name, None))
    for obj in list(bpy.context.scene.objects):
        if obj is not root and obj.parent is None:
            obj.parent = root
    return root


def close_case_lid():
    """The FBX shows the case open; turn the lid about its hinge until it sits on the body."""
    lid = bpy.data.objects[AIRPODS_FBX_LID]
    lid_meshes = [lid, *lid.children_recursive]
    body = bpy.data.objects[AIRPODS_FBX_CASE_BODY]
    body_low, body_high = world_bounds([body, *body.children_recursive])
    hinge = Vector((0, body_high.y - 0.001, body_high.z))
    original = lid.matrix_world.copy()

    def misfit(angle_deg):
        turn = Matrix.Translation(hinge) @ Matrix.Rotation(math.radians(angle_deg), 4, "X") @ Matrix.Translation(-hinge)
        lid.matrix_world = turn @ original
        bpy.context.view_layer.update()
        low, high = world_bounds(lid_meshes)
        return abs(low.y - body_low.y) + abs(high.y - body_high.y) + abs(low.z - body_high.z)

    best = min(range(-180, 181), key=misfit)
    misfit(best)
    print(f"case lid closed by turning {best} deg")


def import_airpods_part(model, fbx):
    bpy.ops.import_scene.fbx(filepath=str(fbx))
    group = bpy.data.objects[AIRPODS_FBX_GROUPS[model]]
    keep = {group, *group.children_recursive}
    if model == "airpods_case":
        close_case_lid()
    for obj in [o for o in bpy.data.objects if o.type == "MESH" and o not in keep]:
        bpy.data.objects.remove(obj)
    return wrap_scene(model)


def iphone_material(textures):
    """The glTF from the COLLADA keeps only UVs; wire the Sketchfab PBR maps back in."""
    material = bpy.data.materials.new("iphone")
    nodes, links = material.node_tree.nodes, material.node_tree.links
    bsdf = nodes["Principled BSDF"]

    def texture(file, is_color):
        node = nodes.new("ShaderNodeTexImage")
        node.image = bpy.data.images.load(str(textures / file))
        node.image.colorspace_settings.name = "sRGB" if is_color else "Non-Color"
        return node

    links.new(texture("Material_albedo.jpg", True).outputs["Color"], bsdf.inputs["Base Color"])
    links.new(texture("Material_metallic.jpg", False).outputs["Color"], bsdf.inputs["Metallic"])
    links.new(texture("Material_roughness.jpg", False).outputs["Color"], bsdf.inputs["Roughness"])
    # Material_opacity.jpg is left out: wired to Alpha it lets the screen show
    # through the back and the camera through the front
    links.new(texture("Material_emissive.jpg", True).outputs["Color"], bsdf.inputs["Emission Color"])
    # The emissive map also lights the Apple logo on the back; only faces
    # pointing out of the screen side (+Y in the file) may glow
    coords = nodes.new("ShaderNodeTexCoord")
    facing = nodes.new("ShaderNodeSeparateXYZ")
    is_screen_side = nodes.new("ShaderNodeMath")
    is_screen_side.operation = "GREATER_THAN"
    is_screen_side.inputs[1].default_value = 0.5
    links.new(coords.outputs["Normal"], facing.inputs[0])
    links.new(facing.outputs["Y"], is_screen_side.inputs[0])
    screen_glow = nodes.new("ShaderNodeMath")
    screen_glow.operation = "MULTIPLY"
    screen_glow.inputs[1].default_value = SCREEN_GLOW
    links.new(is_screen_side.outputs[0], screen_glow.inputs[0])
    links.new(screen_glow.outputs[0], bsdf.inputs["Emission Strength"])
    normal_map = nodes.new("ShaderNodeNormalMap")
    links.new(texture("Material_normal.png", False).outputs["Color"], normal_map.inputs["Color"])
    links.new(normal_map.outputs["Normal"], bsdf.inputs["Normal"])
    return material


def import_iphone(glb, textures):
    bpy.ops.import_scene.gltf(filepath=str(glb))
    material = iphone_material(textures)
    for obj in bpy.context.scene.objects:
        if obj.type == "MESH":
            obj.data.materials.clear()
            obj.data.materials.append(material)

    # The metal back and the polished logo mirror the key light into a white
    # blot, and the rim light behind leaks through the logo onto the screen.
    # The key still lights the phone, it just stays out of its reflections.
    bpy.data.objects.remove(bpy.data.objects["rim"])
    bpy.data.objects["key"].visible_glossy = False
    # The glTF importer already turns the file's Y-up into Blender's Z-up,
    # which stands the phone upright. The file is in centimetres as plain
    # units, 36 tall; scaled down so the studio lights stay outside the model.
    root = wrap_scene("iphone")
    root.scale = (IPHONE_GLB_SCALE,) * 3
    return root


MODELS = {
    "airpod_left": lambda: build_airpod(is_left=True),
    "airpod_right": lambda: build_airpod(is_left=False),
    "airpods_case": build_case,
    "iphone": build_iphone,
}

# Turn that faces the camera in frame 0: the grille side of a pod towards the
# middle of the widget, the case and phone front-on.
FRONT_TURN_DEG = {"airpod_left": -30, "airpod_right": 30, "airpods_case": 0, "iphone": 0}
# The imported phone is modelled with its back towards -Y
IPHONE_GLB_SCALE = 0.01
# The display has to read on the widget's dark card at icon size
SCREEN_GLOW = 1.8
IMPORTED_IPHONE_FRONT_TURN_DEG = 180


# --- scene -------------------------------------------------------------------

def area_light(name, location, size, energy, color=(1, 1, 1)):
    data = bpy.data.lights.new(name, "AREA")
    data.size, data.energy, data.color = size, energy, color
    light = link(bpy.data.objects.new(name, data))
    light.location = location
    light.rotation_euler = (Vector((0, 0, 0)) - Vector(location)).to_track_quat("-Z", "Y").to_euler()
    return light


def setup_scene(size):
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = SAMPLES
    scene.cycles.use_denoising = True
    prefs = bpy.context.preferences.addons["cycles"].preferences
    for device_type in ("OPTIX", "CUDA"):
        try:
            prefs.compute_device_type = device_type
            prefs.get_devices()
            break
        except TypeError:
            continue
    for device in prefs.devices:
        device.use = True
    scene.cycles.device = "GPU"
    scene.render.film_transparent = True
    scene.render.resolution_x = scene.render.resolution_y = size
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    scene.view_settings.view_transform = "Standard"
    scene.view_settings.look = "None"

    world = bpy.data.worlds.new("studio")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.32, 0.33, 0.36, 1)
    world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.6
    scene.world = world

    area_light("key", (-9, -12, 10), 9, 2400)
    area_light("fill", (12, -8, 2), 12, 900, (0.9, 0.95, 1.0))
    area_light("rim", (4, 12, 8), 6, 1600)
    area_light("top", (0, 0, 16), 14, 700)


def setup_camera(model):
    meshes = [obj for obj in [model, *model.children_recursive] if obj.type == "MESH"]
    corners = [obj.matrix_world @ Vector(corner) for obj in meshes for corner in obj.bound_box]
    # The middle of the outer bounds, not the mean of all corners: a model made
    # of many small parts (grilles, sensors) would pull the mean towards them
    low, high = world_bounds(meshes)
    center = (low + high) / 2
    # Every turn must fit: the horizontal reach is the largest distance from the turning axis
    reach = max(math.hypot(c.x - center.x, c.y - center.y) for c in corners)
    half_height = max(abs(c.z - center.z) for c in corners)

    data = bpy.data.cameras.new("camera")
    data.type = "ORTHO"
    elevation = math.radians(CAMERA_ELEVATION_DEG)
    data.ortho_scale = 2 * max(reach, half_height * math.cos(elevation) + reach * math.sin(elevation)) * FRAME_MARGIN
    camera = link(bpy.data.objects.new("camera", data))
    camera.location = center + Vector((0, -math.cos(elevation), math.sin(elevation))) * CAMERA_DISTANCE
    camera.rotation_euler = (center - camera.location).to_track_quat("-Z", "Y").to_euler()
    bpy.context.scene.camera = camera
    return center


def center_front_view(model):
    """Moves the camera so the resting front view sits exactly in the middle.

    A pod is lopsided around its turning axis (bud on one side, stem on the
    other), so centring the axis leaves the front view off-centre. The widget
    shows the front view almost all the time; the turn may wander a little.
    """
    camera = bpy.context.scene.camera
    right, up, _ = camera.matrix_world.to_3x3().transposed()
    depsgraph = bpy.context.evaluated_depsgraph_get()
    along_right, along_up = [], []
    for obj in [o for o in [model, *model.children_recursive] if o.type == "MESH"]:
        evaluated = obj.evaluated_get(depsgraph)
        mesh = evaluated.to_mesh()
        for vertex in mesh.vertices:
            point = obj.matrix_world @ vertex.co
            along_right.append(point.dot(right))
            along_up.append(point.dot(up))
        evaluated.to_mesh_clear()
    shift_right = (min(along_right) + max(along_right)) / 2 - camera.location.dot(right)
    shift_up = (min(along_up) + max(along_up)) / 2 - camera.location.dot(up)
    camera.location += right * shift_right + up * shift_up
    # Widen the frame by the shift, so the turn still fits on the far side
    turn_fit = camera.data.ortho_scale + 2 * max(abs(shift_right), abs(shift_up))
    front_size = math.sqrt((max(along_right) - min(along_right)) * (max(along_up) - min(along_up)))
    camera.data.ortho_scale = max(turn_fit, front_size / FRONT_VIEW_FILL)


def smoothstep(share):
    """Ease in and out: starts and stops at zero speed, fastest halfway."""
    return share * share * (3 - 2 * share)


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", choices=MODELS, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--size", type=int, default=160)
    parser.add_argument("--angles", help="comma-separated degrees for a preview instead of the full turn")
    parser.add_argument("--airpods-fbx", type=Path, help="AirPods 4 model; without it the pods are built from primitives")
    parser.add_argument("--iphone-glb", type=Path, help="iPhone model; needs --iphone-textures")
    parser.add_argument("--iphone-textures", type=Path)
    return parser.parse_args(sys.argv[sys.argv.index("--") + 1:])


def main():
    args = parse_args()
    bpy.ops.wm.read_factory_settings(use_empty=True)
    setup_scene(args.size)
    if args.airpods_fbx and args.model in AIRPODS_FBX_GROUPS:
        model = import_airpods_part(args.model, args.airpods_fbx)
    elif args.iphone_glb and args.model == "iphone":
        model = import_iphone(args.iphone_glb, args.iphone_textures)
    else:
        model = MODELS[args.model]()
    bpy.context.view_layer.update()
    center = setup_camera(model)

    # Turn around the model's own vertical axis through its centre
    pivot = link(bpy.data.objects.new("pivot", None))
    pivot.location = center
    model.parent = pivot
    model.location -= center
    is_imported_iphone = bool(args.iphone_glb) and args.model == "iphone"
    front_turn = IMPORTED_IPHONE_FRONT_TURN_DEG if is_imported_iphone else FRONT_TURN_DEG[args.model]
    base_turn = math.radians(front_turn)
    pivot.rotation_euler = (0, 0, base_turn)
    bpy.context.view_layer.update()
    center_front_view(model)

    angles = [float(a) for a in args.angles.split(",")] if args.angles else [
        360 * smoothstep(frame / FULL_TURN_FRAMES) for frame in range(FULL_TURN_FRAMES + 1)]
    args.out.mkdir(parents=True, exist_ok=True)
    for index, angle in enumerate(angles):
        pivot.rotation_euler = (0, 0, base_turn + math.radians(angle))
        bpy.context.scene.render.filepath = str(args.out / f"{index:03d}.png")
        bpy.ops.render.render(write_still=True)


main()
