# old gods architecture diagram

+------------------------------------------------------------------------------+
|Engine                                                                        |
|                                             +---------+ +---------+          |
|                                             |Rendering| |Scripting|          |
|                                             +---^-----+ +-^----+--+          |
|                                                 |         |    |             |
+------------------------------------------------------------------------------+
                                                  |         |    |
+----------------------+                          |         |    |
|                      |                      +---+---------+----v-------------+
|TiledmapSystem       <--+LoadMap component<---+ Specs entity component system |
|                     X|                      |         XXXX                   |
|* loads Tiled maps   --->Tiledmap component+-->     XXXXXXXXXX                |
|* breaks down maps    |                      |     XXX      XXX               |
|  into base           |                      |    XXX      XXXX               |
|  components         <--+Tiledmap component<--+   XXX XXXXXXXXX               |
|                     X|                      |    XXX  XXXXX                  |
|                     --->Object+-------------->    XXX          XXX           |
+----------------------+  JSON                |      XXXXXXXXXXXXXX            |
                          Position            |       XXXXXXXXXX               |
                          Rendering           |                                |
                          Shape               |         XXXXXX                 |
                          Barrier             |       XXXXXXXXXXX              |
                          OriginOffset        |      XXXX      XXX             |
                          Animation           |      XXXX                      |
                          Fence, StepFence    |      XXX          XXX          |
                          Zone                |       XXX         XXX          |
                          etc.                |       XXXX     XXXXX           |
                          (base components)   |         XXXXXXXXXX             |
                                              |                                |
+----------------------+                      |         XXXXX                  |
|PhysicsSystem         |                      |       XXXXXXXXX                |
|                     <--+Position<------------+      XXX    XXX               |
|* moves things based  |  Velocity            |       XXX                      |
|  on Position,        |  Shape               |        XXXXXXXXXXXXXXX         |
|  Velocity, Shape,    |  Barrier             |          XXXXXXXXXXXXXX        |
|  Barrier comps       |                      |                      XXX       |
|* updates AABBTree    |                      |       XXX            XXX       |
|  resource used to    |                      |        XXX          XXX        |
|  query the map       |                      |        XXXXXXXXXXXXXX          |
|                      |                      |           XXXXXXXXX            |
+----------------------+                      |                                |
        |FPSCounter    |                      |                                |
        +----------+---+                      |                                |
        | AABBTree |                          |                                |
        +----------+                          |                                |
                                              |                                |
                                              |                                |
+---------------------+                       |                                |
|ScreenSystem         <--+Player<--------------+                               |
|                     |   Position            |                                |
|* follows players,   |                       |                                |
|  maintaining a      |                       |                                |
|  Screen resource    |                       |                                |
|  used to map coords |                       |                                |
|  from the map       |                       |                                |
|                     |                       |                                |
+------+--------------+                       |                                |
       | Screen       |                       |                                |
       +--------------+                       |                                |
                                              |                                |
                                              |                                |
                                              |                                |
+------------------+                          |                                |
|AnimationSystem   <-----+Animation<-----------+                               |
|* progresses      |                          |                                |
|  Animations      +----->Rendering+----------->                               |
|                  |                          |                                |
+------------------+                          |                                |
                                              |                                |
                                              |                                |
                                              |                                |
                                              |                                |
+------------------+                          |                                |
|GamepadSystem     <----+Player<---------------+                               |
|* updates input   |                          |                                |
|  controller      |                          |                                |
|  state           |                          |                                |
|                  |                          |                                |
+------------------+                          |                                |
|PlayerControllers |                          |                                |
+------------------+                          |                                |
                                              |                                |
                                              |                                |
                                              |                                |
+------------------+                          |                                |
|PlayerSystem      <----+Object<---------------+                               |
|* creates Player  |                          |                                |
|  components      +---->Player+--------------->                               |
|* updates player  |                          |                                |
|  character       |                          |                                |
|  velocities based|                          |                                |
|  on player input |                          |                                |
+------------------+                          |                                |
                                              |                                |
                                              |                                |
+------------------+                          |                                |
|ZoneSystem        <----+Zone<-----------------+                               |
|* updates zones   |                          |                                |
|                  |                          |                                |
+------------------+                          |                                |
          |AABBTree|                          |                                |
          +--------+                          |                                |
                                              |                                |
                                              |                                |
                                              |                                |
+------------------+                          |                                |
|FenceSystem       |                          |                                |
|* updates fences  <----+[Step]Fence<----------+                               |
|  and step fences |                          +--------------------------------+
|                  |
+------------------+
          |AABBTree|
          +--------+
