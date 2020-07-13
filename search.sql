delimiter // 
create function search(terms varchar(240)) returns int deterministic
begin
	declare id int;
	select quiz.id into id from quiz where match(name, description) against (terms in boolean mode);
	return id; 
end //
delimiter ;
