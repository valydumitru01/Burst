import math
import pygame
import moderngl
import glm
import numpy as np

AIR=0
STONE=1
DIRT=2
GRASS=3
WOOD=4
LEAVES=5
PLANK=6
ROOF=7

def mat4_bytes(m):
    return np.asarray(m.to_list(),dtype=np.float32).tobytes()

def vec3_bytes(v):
    return np.asarray((v.x,v.y,v.z),dtype=np.float32).tobytes()

class Noise3D:
    def __init__(self, seed):
        self.seed = np.uint32(seed)

    @staticmethod
    def _u32(x):
        return np.asarray(x, dtype=np.uint32)

    def hash(self, x, y, z):
        xi = self._u32(np.floor(x))
        yi = self._u32(np.floor(y))
        zi = self._u32(np.floor(z))

        h = (
                xi * np.uint32(374761393) +
                yi * np.uint32(668265263) +
                zi * np.uint32(2147483647) +
                self.seed
        )

        h = (h ^ (h >> np.uint32(13))) * np.uint32(1274126177)
        h = h ^ (h >> np.uint32(16))

        return h

    def noise(self, x, y, z):
        # Value noise in [-1,1]
        return self.hash(x, y, z).astype(np.float32) * (2.0 / 4294967295.0) - 1.0

    def fbm(self, x, y, z, oct=5):
        freq = 1.0
        amp = 1.0
        norm = 0.0
        total = 0.0

        for _ in range(oct):
            total += self.noise(x * freq, y * freq, z * freq) * amp
            norm += amp
            freq *= 2.0
            amp *= 0.5

        return total / norm


def smoothstep(a, b, x):
    t = np.clip((x - a) / (b - a), 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)


class Terrain:
    def __init__(self, seed: int, size: int):
        self.size = size
        self.seed = seed

        self.n0 = Noise3D(seed)
        self.n1 = Noise3D(seed + 1)
        self.n2 = Noise3D(seed + 2)

        self.palette = np.zeros((256, 3), np.uint8)
        self.palette[STONE]  = (110, 110, 110)
        self.palette[DIRT]   = (120,  90,  50)
        self.palette[GRASS]  = ( 80, 140,  60)
        self.palette[WOOD]   = ( 90,  60,  30)
        self.palette[LEAVES] = ( 40, 120,  40)
        self.palette[PLANK]  = (160, 130,  90)
        self.palette[ROOF]   = (140,  50,  50)

    # ---------------------------------------------------------------------

    def generate(self, ox: int, oz: int):
        s = self.size

        vox  = np.zeros((s, s, s), np.uint8)
        mats = np.zeros((s, s, s), np.uint8)

        # -----------------------------------------------------------------
        # Coordinate fields (broadcasted, no indices() allocation)
        # -----------------------------------------------------------------

        x = (np.arange(s, dtype=np.float32)[:, None, None] + ox)
        y = (np.arange(s, dtype=np.float32)[None, :, None])
        z = (np.arange(s, dtype=np.float32)[None, None, :] + oz)

        XZx = x[:, 0, :]
        XZz = z[0, 0, :]

        X, Z = np.meshgrid(
            np.arange(s, dtype=np.float32) + ox,
            np.arange(s, dtype=np.float32) + oz,
            indexing="ij"
        )

        Z0 = np.zeros_like(X)

        # -----------------------------------------------------------------
        # Domain warping (2D)
        # -----------------------------------------------------------------

        warp_x = self.n0.fbm(X * 0.008, Z0, Z * 0.008, 4) * 20.0
        warp_z = self.n1.fbm(X * 0.008, Z0, Z * 0.008, 4) * 20.0

        # -----------------------------------------------------------------
        # Heightmap with real mountains
        # -----------------------------------------------------------------

        continental = self.n0.fbm(
            (X + warp_x) * 0.004, Z0, (Z + warp_z) * 0.004, 5
        ) * 0.5 + 0.5

        ridged = 1.0 - np.abs(self.n1.fbm(
            (X + warp_x) * 0.012, Z0, (Z + warp_z) * 0.012, 5
        ))
        ridged **= 3.5

        mountain_mask = smoothstep(0.45, 0.75, ridged)
        mountains = ridged * mountain_mask * (s * 0.9)

        detail = self.n2.fbm(
            (X + warp_x) * 0.05, Z0, (Z + warp_z) * 0.05, 2
        ) * 3.0

        height = 6.0 + continental * 18.0 + mountains + detail
        height = np.clip(height, 2, s - 2)

        # -----------------------------------------------------------------
        # Density field (overhangs + strata)
        # -----------------------------------------------------------------

        H = height[:, None, :]

        overhang = self.n1.fbm(x * 0.035, y * 0.035, z * 0.035, 3)
        strata   = self.n2.fbm(x * 0.07,  y * 0.07,  z * 0.07,  2)

        density = (H - y) \
                  + overhang * mountain_mask[:, None, :] * 6.0 \
                  + strata * 2.0

        solid = density > 0.0

        # -----------------------------------------------------------------
        # Caves: caverns + tunnels
        # -----------------------------------------------------------------

        cavern = self.n0.fbm(x * 0.04, y * 0.04, z * 0.04, 2) * 0.5 + 0.5
        worm   = np.abs(self.n1.fbm(x * 0.13, y * 0.13, z * 0.13, 3))

        cave_mask = (
                ((cavern > 0.78) | (worm < 0.08)) &
                (y > 4) &
                (y < H - 2)
        )

        solid[cave_mask] = False

        vox[solid]  = 1
        mats[solid] = STONE

        # -----------------------------------------------------------------
        # Surface detection (vectorized)
        # -----------------------------------------------------------------

        has_solid = solid.any(axis=1)
        top = (s - 1) - solid[:, ::-1, :].argmax(axis=1)
        top[~has_solid] = -1

        # slope from heightmap
        gx, gz = np.gradient(height)
        slope = np.maximum(np.abs(gx), np.abs(gz))

        steep = slope > 1.8

        # soil depth: thinner on mountains and cliffs
        soil = (2 + (1.0 - mountain_mask) * 4.0).astype(np.int32)
        soil -= steep.astype(np.int32) * soil
        soil = np.clip(soil, 0, 6)

        Y = np.arange(s)[None, :, None]
        T = top[:, None, :]

        surface = (Y == T) & (T >= 0)
        subsurf = (Y < T) & (Y >= (T - soil))

        grass_cols = (~steep) & (mountain_mask < 0.6)

        mats[surface & grass_cols[:, None, :]] = GRASS
        mats[subsurf & grass_cols[:, None, :]] = DIRT

        # -----------------------------------------------------------------
        # Trees (kept sparse and deterministic)
        # -----------------------------------------------------------------

        for ix in range(5, s - 5, 9):
            for iz in range(5, s - 5, 9):
                iy = top[ix, iz]
                if iy < 5 or iy > s - 12:
                    continue
                if steep[ix, iz] or mountain_mask[ix, iz] > 0.6:
                    continue

                r = self.n0.hash(ix + ox, iy, iz + oz) & 255
                if r < 28:
                    h = 6 + (r & 3)
                    vox[ix, iy + 1:iy + h, iz] = 1
                    mats[ix, iy + 1:iy + h, iz] = WOOD

                    rx = slice(ix - 2, ix + 3)
                    ry = slice(iy + h - 2, iy + h + 3)
                    rz = slice(iz - 2, iz + 3)

                    mask = (
                                   (np.arange(rx.start, rx.stop)[:, None, None] - ix) ** 2 +
                                   (np.arange(ry.start, ry.stop)[None, :, None] - (iy + h)) ** 2 +
                                   (np.arange(rz.start, rz.stop)[None, None, :] - iz) ** 2
                           ) <= 6

                    vox[rx, ry, rz][mask] = 1
                    mats[rx, ry, rz][mask] = LEAVES

        # -----------------------------------------------------------------

        col = self.palette[mats]
        col[vox == 0] = 0
        return vox, col

class VoxelOctree:
    def __init__(self,occ,col):
        self.occ=[]
        self.col=[]
        o=occ
        c=col.astype(np.float32)
        s=o.shape[0]
        while True:
            self.occ.append(o)
            self.col.append(c.astype(np.uint8))
            if s==1: break
            no=np.zeros((s//2,s//2,s//2),dtype=np.uint8)
            nc=np.zeros((s//2,s//2,s//2,3),dtype=np.float32)
            for dx in (0,1):
                for dy in (0,1):
                    for dz in (0,1):
                        bo=o[dx::2,dy::2,dz::2]
                        bc=c[dx::2,dy::2,dz::2]
                        no+=bo
                        nc+=bc*bo[...,None]
            o=(no>=4).astype(np.uint8)
            c=nc/np.maximum(no,1)[...,None]
            s//=2

class Chunk:
    def __init__(self,cx,cz,size,o,c):
        self.origin=glm.vec3(cx*size,0,cz*size)
        h=size*0.5
        self.center=glm.vec3(self.origin.x+h,h,self.origin.z+h)
        self.size=size
        self.oct=VoxelOctree(o,c)
        self.to=[]
        self.tc=[]

    def upload(self,ctx):
        if self.to: return
        for o,c in zip(self.oct.occ,self.oct.col):
            to=ctx.texture3d(o.shape,1,o.tobytes(),dtype="u1")
            tc=ctx.texture3d(c.shape[:3],3,c.tobytes(),dtype="u1")
            to.filter=tc.filter=(moderngl.NEAREST,moderngl.NEAREST)
            self.to.append(to)
            self.tc.append(tc)

    def d2(self,p):
        d=p-self.center
        return d.x*d.x+d.y*d.y+d.z*d.z

    def render(self,prog,vao,l):
        self.to[l].use(0)
        self.tc[l].use(1)
        prog["u_occ"].value=0
        prog["u_col"].value=1
        prog["u_level"].value=l
        prog["u_base_size"].value=self.size
        prog["u_chunk_origin"].write(vec3_bytes(self.origin))
        vao.render(instances=(self.size>>l)**3)

class Camera:
    def __init__(self,p,a):
        self.p=p
        self.y=-90
        self.x=0
        self.a=a
        self.f=glm.vec3(0,0,-1)
        self.u=glm.vec3(0,1,0)
        self.r=glm.vec3(1,0,0)
        self._u()

    def _u(self):
        cy,sy=math.cos(math.radians(self.y)),math.sin(math.radians(self.y))
        cp,sp=math.cos(math.radians(self.x)),math.sin(math.radians(self.x))
        self.f=glm.normalize(glm.vec3(cy*cp,sp,sy*cp))
        self.r=glm.normalize(glm.cross(self.f,glm.vec3(0,1,0)))
        self.u=glm.normalize(glm.cross(self.r,self.f))

    def step(self,dt,k,mx,my):
        self.y+=mx*0.1
        self.x=max(-89,min(89,self.x-my*0.1))
        self._u()
        v=glm.vec3(0)
        if k[pygame.K_w]: v+=self.f
        if k[pygame.K_s]: v-=self.f
        if k[pygame.K_a]: v-=self.r
        if k[pygame.K_d]: v+=self.r
        if k[pygame.K_SPACE]: v+=self.u
        if k[pygame.K_LCTRL]: v-=self.u
        if glm.length(v)>0: self.p+=glm.normalize(v)*dt*20

    def view(self):
        return glm.lookAt(self.p,self.p+self.f,self.u)

    def proj(self):
        return glm.perspective(glm.radians(70),self.a,0.1,2000)

def lod(d):
    if d<80**2:return 0
    if d<160**2:return 1
    if d<320**2:return 2
    if d<640**2:return 3
    if d<1000**2:return 4
    return 5

pygame.init()
w,h=1280,720
pygame.display.set_mode((w,h),pygame.OPENGL|pygame.DOUBLEBUF)
pygame.event.set_grab(True)
pygame.mouse.set_visible(False)

ctx=moderngl.create_context()
ctx.enable(moderngl.DEPTH_TEST|moderngl.CULL_FACE)

prog=ctx.program(
    vertex_shader="""
#version 330
in vec3 in_pos;
in vec3 in_norm;
uniform mat4 u_vp;
uniform vec3 u_chunk_origin;
uniform usampler3D u_occ;
uniform usampler3D u_col;
uniform int u_level;
uniform int u_base_size;
out vec3 v_col;
out vec3 v_norm;
void main(){
 int cube=1<<u_level;
 int dim=u_base_size>>u_level;
 int id=gl_InstanceID;
 ivec3 c=ivec3(id/(dim*dim),(id/dim)%dim,id%dim);
 if(texelFetch(u_occ,c,0).r==0u){gl_Position=vec4(2);return;}
 vec3 wp=vec3(c*cube)+in_pos*float(cube)+u_chunk_origin;
 gl_Position=u_vp*vec4(wp,1);
 v_col=vec3(texelFetch(u_col,c,0).rgb)/255.0;
 v_norm=in_norm;
}
""",
    fragment_shader="""
#version 330
in vec3 v_col;
in vec3 v_norm;
out vec4 f;
void main(){
 float d=max(dot(normalize(v_norm),normalize(vec3(0.4,1,0.2))),0.2);
 f=vec4(v_col*d,1);
}
"""
)

cube=np.array([
    [0,0,1,0,0,1],[1,0,1,0,0,1],[1,1,1,0,0,1],
    [0,0,1,0,0,1],[1,1,1,0,0,1],[0,1,1,0,0,1],
    [1,0,0,0,0,-1],[0,0,0,0,0,-1],[0,1,0,0,0,-1],
    [1,0,0,0,0,-1],[0,1,0,0,0,-1],[1,1,0,0,0,-1],
    [0,0,0,-1,0,0],[0,0,1,-1,0,0],[0,1,1,-1,0,0],
    [0,0,0,-1,0,0],[0,1,1,-1,0,0],[0,1,0,-1,0,0],
    [1,0,1,1,0,0],[1,0,0,1,0,0],[1,1,0,1,0,0],
    [1,0,1,1,0,0],[1,1,0,1,0,0],[1,1,1,1,0,0],
    [0,1,1,0,1,0],[1,1,1,0,1,0],[1,1,0,0,1,0],
    [0,1,1,0,1,0],[1,1,0,0,1,0],[0,1,0,0,1,0],
    [0,0,0,0,-1,0],[1,0,0,0,-1,0],[1,0,1,0,-1,0],
    [0,0,0,0,-1,0],[1,0,1,0,-1,0],[0,0,1,0,-1,0]
],dtype=np.float32)

vao=ctx.vertex_array(prog,[(ctx.buffer(cube.tobytes()),"3f 3f","in_pos","in_norm")])

terrain=Terrain(1337,32)
chunks=[]
for z in range(16):
    for x in range(16):
        o,c=terrain.generate(x*32,z*32)
        ch=Chunk(x,z,32,o,c)
        ch.upload(ctx)
        chunks.append(ch)

cam=Camera(glm.vec3(256,50,256),w/h)
clock=pygame.time.Clock()
while True:
    dt=clock.tick()/1000
    for e in pygame.event.get():
        if e.type==pygame.QUIT: exit()
    mx,my=pygame.mouse.get_rel()
    k=pygame.key.get_pressed()
    if k[pygame.K_ESCAPE]: exit()
    cam.step(dt,k,mx,my)

    ctx.clear(0.5,0.7,1)
    vp=cam.proj()*cam.view()
    prog["u_vp"].write(mat4_bytes(vp))
    for ch in chunks:
        l=lod(ch.d2(cam.p))
        ch.render(prog,vao,l)
    pygame.display.flip()


