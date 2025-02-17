# RmDb
### Project in Rust with Rocket/SQLx/Tera

The idea of the project is to have a database with movies, where users can give submit their ratings.

Might add more features later...

- [x] ~~Create DB~~
- [x] ~~Create Home template~~
- [x] ~~Add Movies to DB~~
- [x] ~~Create CRUD for Movies~~
- [x] ~~Add templates for Movies (create/edit/delete)~~
- [x] ~~Add Users to DB~~
- [x] ~~Create auth for Users~~
- [ ] Add templates for Users (Sign/login/logout)
- [ ] Add ratings to DB (user <--> movie)
- [ ] Add to templates the ability to rank the movies

### Instructions to run:

1. Run ```docker compose up```
2. Run migrations ```sqlx migrate run``` (requires ```sqlx-cli```)
3. Create ```.env``` file (use ```.env.example``` for reference)
4. Run project ``` cargo run ```

</br>

<p align="center">
    <img align="center" alt="ferris" width="400" src="https://github.com/Axl-91/rmdb/blob/main/static/images/ferris.png">
</p>
