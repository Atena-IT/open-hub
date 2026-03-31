import os
import re

def update_file(path):
    with open(path, 'r') as f:
        content = f.read()

    # We only want to replace within route("...")
    # Find all route(...)
    def repl_route(m):
        route_str = m.group(1)
        # replace :param with {param}
        route_str = re.sub(r':([a-zA-Z_]+)', r'{\1}', route_str)
        # replace *path with {*path}
        route_str = re.sub(r'\*([a-zA-Z_]+)', r'{*\1}', route_str)
        return f'.route("{route_str}"'

    new_content = re.sub(r'\.route\("([^"]+)"', repl_route, content)
    
    with open(path, 'w') as f:
        f.write(new_content)

update_file('crates/cas-server/src/router.rs')
update_file('crates/hub-api/src/routes/mod.rs')
update_file('crates/web-ui/src/lib.rs')

